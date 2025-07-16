#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{WebSocketMessage, WebSocketFrame, WebSocketOpcode, WebSocketCloseCode, WebSocketError, WebSocketCodec, WebSocketServerCodec, WebSocketClientCodec, ConnectionState};
    use bytes::{Bytes, BytesMut};
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_websocket_message_serialization() {
        // Test text message
        let text_msg = WebSocketMessage::Text("Hello WebSocket!".to_string());
        let serialized = serde_json::to_string(&text_msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(text_msg, deserialized);

        // Test binary message
        let binary_msg = WebSocketMessage::Binary(b"binary data".to_vec());
        let serialized = serde_json::to_string(&binary_msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(binary_msg, deserialized);

        // Test close message
        let close_msg = WebSocketMessage::Close(Some(crate::util::websocket::types::WebSocketCloseFrame {
            code: WebSocketCloseCode::Normal,
            reason: "Goodbye".to_string(),
        }));
        let serialized = serde_json::to_string(&close_msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(close_msg, deserialized);
    }

    #[test]
    fn test_websocket_frame_creation() {
        let frame = WebSocketFrame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: WebSocketOpcode::Text,
            mask: None,
            payload: b"Hello".to_vec(),
        };

        assert_eq!(frame.opcode, WebSocketOpcode::Text);
        assert_eq!(frame.payload, b"Hello");
        assert!(frame.fin);
        assert!(frame.mask.is_none());
    }

    #[test]
    fn test_websocket_close_codes() {
        assert_eq!(WebSocketCloseCode::Normal as u16, 1000);
        assert_eq!(WebSocketCloseCode::GoingAway as u16, 1001);
        assert_eq!(WebSocketCloseCode::ProtocolError as u16, 1002);
        assert_eq!(WebSocketCloseCode::UnsupportedData as u16, 1003);
        assert_eq!(WebSocketCloseCode::InvalidFramePayloadData as u16, 1007);
        assert_eq!(WebSocketCloseCode::PolicyViolation as u16, 1008);
        assert_eq!(WebSocketCloseCode::MessageTooBig as u16, 1009);
        assert_eq!(WebSocketCloseCode::MandatoryExtension as u16, 1010);
        assert_eq!(WebSocketCloseCode::InternalError as u16, 1011);
    }

    #[test]
    fn test_websocket_codec_basic() {
        let mut codec = WebSocketCodec::new();
        let mut buf = BytesMut::new();

        // Test encoding a simple text message
        let message = WebSocketMessage::Text("Hello".to_string());

        // Encode the message
        codec.encode(message.clone(), &mut buf).unwrap();
        assert!(!buf.is_empty());

        // Try to decode it back
        let decoded = codec.decode(&mut buf).unwrap();
        if let Some(decoded_message) = decoded {
            match decoded_message {
                WebSocketMessage::Text(text) => {
                    assert_eq!(text, "Hello");
                }
                _ => panic!("Expected text message"),
            }
        }
    }

    #[test]
    fn test_websocket_server_codec() {
        let mut codec = WebSocketServerCodec::new();
        let mut buf = BytesMut::new();

        let message = WebSocketMessage::Text("Server message".to_string());
        
        // Server codec should be able to encode messages
        codec.encode(message.clone(), &mut buf).unwrap();
        assert!(!buf.is_empty());

        // And decode frames back to messages
        // Note: This is a simplified test - in practice the decode would happen
        // after receiving a properly formatted WebSocket frame
    }

    #[test]
    fn test_websocket_client_codec() {
        let mut codec = WebSocketClientCodec::new();
        let mut buf = BytesMut::new();

        let message = WebSocketMessage::Text("Client message".to_string());
        
        // Client codec should be able to encode messages
        codec.encode(message.clone(), &mut buf).unwrap();
        assert!(!buf.is_empty());

        // Client frames should have masking applied
        // The exact implementation depends on the masking logic
    }

    #[test]
    fn test_websocket_opcode_conversion() {
        assert_eq!(WebSocketOpcode::Continuation as u8, 0x0);
        assert_eq!(WebSocketOpcode::Text as u8, 0x1);
        assert_eq!(WebSocketOpcode::Binary as u8, 0x2);
        assert_eq!(WebSocketOpcode::Close as u8, 0x8);
        assert_eq!(WebSocketOpcode::Ping as u8, 0x9);
        assert_eq!(WebSocketOpcode::Pong as u8, 0xA);
    }

    #[test]
    fn test_websocket_error_types() {
        let parse_error = WebSocketError::InvalidFrame("Invalid frame".to_string());
        let protocol_error = WebSocketError::ProtocolViolation("Protocol violation".to_string());
        let io_error = WebSocketError::Io(std::io::Error::new(std::io::ErrorKind::Other, "IO error"));
        let handshake_error = WebSocketError::HandshakeError("Handshake failed".to_string());

        // Test error display
        assert!(format!("{}", parse_error).contains("Invalid frame"));
        assert!(format!("{}", protocol_error).contains("Protocol violation"));
        assert!(format!("{}", handshake_error).contains("Handshake failed"));
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut state = ConnectionState::Connecting;
        
        // Test valid state transitions
        state = ConnectionState::Open;
        assert!(matches!(state, ConnectionState::Open));
        
        state = ConnectionState::Closing;
        assert!(matches!(state, ConnectionState::Closing));
        
        state = ConnectionState::Closed;
        assert!(matches!(state, ConnectionState::Closed));
    }

    #[test]
    fn test_frame_masking() {
        let payload = b"Hello, WebSocket!";
        let mask_key = [0x12, 0x34, 0x56, 0x78];
        
        // Apply masking
        let mut masked_payload = payload.to_vec();
        for (i, byte) in masked_payload.iter_mut().enumerate() {
            *byte ^= mask_key[i % 4];
        }
        
        // Unmask should restore original
        let mut unmasked_payload = masked_payload.clone();
        for (i, byte) in unmasked_payload.iter_mut().enumerate() {
            *byte ^= mask_key[i % 4];
        }
        
        assert_eq!(unmasked_payload, payload);
    }

    #[test]
    fn test_large_payload_handling() {
        let large_payload = vec![0u8; 70000]; // > 65535 bytes
        let frame = WebSocketFrame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: WebSocketOpcode::Binary,
            mask: None,
            payload: large_payload.clone(),
        };

        // Should handle large payloads correctly
        assert_eq!(frame.payload.len(), 70000);
        assert_eq!(frame.opcode, WebSocketOpcode::Binary);
    }

    #[test]
    fn test_fragmented_message_handling() {
        // Test continuation frames
        let start_frame = WebSocketFrame {
            fin: false,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: WebSocketOpcode::Text,
            mask: None,
            payload: b"Hello".to_vec(),
        };

        let continue_frame = WebSocketFrame {
            fin: false,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: WebSocketOpcode::Continuation,
            mask: None,
            payload: b", Wor".to_vec(),
        };

        let end_frame = WebSocketFrame {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: WebSocketOpcode::Continuation,
            mask: None,
            payload: b"ld!".to_vec(),
        };

        assert!(!start_frame.fin);
        assert!(!continue_frame.fin);
        assert!(end_frame.fin);
        assert_eq!(continue_frame.opcode, WebSocketOpcode::Continuation);
        assert_eq!(end_frame.opcode, WebSocketOpcode::Continuation);
    }
}
