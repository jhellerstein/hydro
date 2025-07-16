//! WebSocket codec for parsing and encoding frames.

use std::io;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use rand::Rng;

use crate::util::HttpCodec;
use super::types::{WebSocketFrame, WebSocketMessage, WebSocketOpcode, WebSocketError, WebSocketCloseCode, WebSocketCloseFrame};

/// WebSocket codec for parsing and encoding frames.
#[derive(Debug, Clone)]
pub struct WebSocketCodec {
    /// Maximum frame size (default: 16MB)
    max_frame_size: usize,
    /// Whether this is a server-side codec (affects masking)
    is_server: bool,
    /// Current state of the codec
    state: CodecState,
}

/// WebSocket server codec that handles HTTP upgrade and WebSocket frames.
#[derive(Debug, Clone)]
pub struct WebSocketServerCodec {
    /// HTTP codec for handling the initial handshake
    http_codec: HttpCodec,
    /// WebSocket codec for handling frames after upgrade
    ws_codec: Option<WebSocketCodec>,
}

/// WebSocket client codec that handles HTTP upgrade and WebSocket frames.
#[derive(Debug, Clone)]
pub struct WebSocketClientCodec {
    /// HTTP codec for handling the initial handshake
    http_codec: HttpCodec,
    /// WebSocket codec for handling frames after upgrade
    ws_codec: Option<WebSocketCodec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodecState {
    /// Ready to parse a new frame header
    Header,
    /// Reading payload data
    Payload { 
        header: FrameHeader,
        bytes_remaining: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrameHeader {
    fin: bool,
    rsv1: bool,
    rsv2: bool,
    rsv3: bool,
    opcode: WebSocketOpcode,
    masked: bool,
    payload_len: u64,
    mask: Option<[u8; 4]>,
}

impl Default for WebSocketCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketCodec {
    /// Create a new WebSocket codec.
    pub fn new() -> Self {
        Self {
            max_frame_size: 16 * 1024 * 1024, // 16MB
            is_server: false,
            state: CodecState::Header,
        }
    }
    
    /// Create a new server-side WebSocket codec.
    pub fn server() -> Self {
        Self {
            max_frame_size: 16 * 1024 * 1024,
            is_server: true,
            state: CodecState::Header,
        }
    }
    
    /// Create a new client-side WebSocket codec.
    pub fn client() -> Self {
        Self {
            max_frame_size: 16 * 1024 * 1024,
            is_server: false,
            state: CodecState::Header,
        }
    }
    
    /// Set the maximum frame size.
    pub fn with_max_frame_size(mut self, max_size: usize) -> Self {
        self.max_frame_size = max_size;
        self
    }
    
    /// Parse frame header from buffer.
    fn parse_header(&self, buf: &mut BytesMut) -> Result<Option<FrameHeader>, WebSocketError> {
        if buf.len() < 2 {
            return Ok(None); // Need more data
        }
        
        let first_byte = buf[0];
        let second_byte = buf[1];
        
        let fin = (first_byte & 0x80) != 0;
        let rsv1 = (first_byte & 0x40) != 0;
        let rsv2 = (first_byte & 0x20) != 0;
        let rsv3 = (first_byte & 0x10) != 0;
        let opcode = WebSocketOpcode::from_u8(first_byte & 0x0F)?;
        
        let masked = (second_byte & 0x80) != 0;
        let mut payload_len = (second_byte & 0x7F) as u64;
        
        // Validate masking based on codec type
        if self.is_server && !masked {
            return Err(WebSocketError::ProtocolViolation(
                "Client frames must be masked".to_string()
            ));
        }
        if !self.is_server && masked {
            return Err(WebSocketError::ProtocolViolation(
                "Server frames must not be masked".to_string()
            ));
        }
        
        // Validate reserved bits
        if rsv1 || rsv2 || rsv3 {
            return Err(WebSocketError::ProtocolViolation(
                "Reserved bits must be 0".to_string()
            ));
        }
        
        let mut header_len = 2;
        
        // Extended payload length
        if payload_len == 126 {
            if buf.len() < header_len + 2 {
                return Ok(None); // Need more data
            }
            payload_len = u16::from_be_bytes([buf[header_len], buf[header_len + 1]]) as u64;
            header_len += 2;
        } else if payload_len == 127 {
            if buf.len() < header_len + 8 {
                return Ok(None); // Need more data
            }
            payload_len = u64::from_be_bytes([
                buf[header_len], buf[header_len + 1], buf[header_len + 2], buf[header_len + 3],
                buf[header_len + 4], buf[header_len + 5], buf[header_len + 6], buf[header_len + 7],
            ]);
            header_len += 8;
        }
        
        // Validate frame size
        if payload_len > self.max_frame_size as u64 {
            return Err(WebSocketError::FrameTooLarge {
                size: payload_len as usize,
                max_size: self.max_frame_size,
            });
        }
        
        // Validate control frame constraints
        if opcode.is_control() {
            if !fin {
                return Err(WebSocketError::ProtocolViolation(
                    "Control frames must not be fragmented".to_string()
                ));
            }
            if payload_len > 125 {
                return Err(WebSocketError::ProtocolViolation(
                    "Control frames must have payload <= 125 bytes".to_string()
                ));
            }
        }
        
        // Read masking key if present
        let mask = if masked {
            if buf.len() < header_len + 4 {
                return Ok(None); // Need more data
            }
            let mask = [buf[header_len], buf[header_len + 1], buf[header_len + 2], buf[header_len + 3]];
            header_len += 4;
            Some(mask)
        } else {
            None
        };
        
        // Remove header from buffer
        buf.advance(header_len);
        
        Ok(Some(FrameHeader {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            masked,
            payload_len,
            mask,
        }))
    }
    
    /// Convert WebSocket frame to message.
    fn frame_to_message(frame: WebSocketFrame) -> Result<WebSocketMessage, WebSocketError> {
        match frame.opcode {
            WebSocketOpcode::Text => {
                let text = String::from_utf8(frame.payload)
                    .map_err(|e| WebSocketError::InvalidUtf8(e))?;
                Ok(WebSocketMessage::Text(text))
            }
            WebSocketOpcode::Binary => {
                Ok(WebSocketMessage::Binary(frame.payload))
            }
            WebSocketOpcode::Ping => {
                Ok(WebSocketMessage::Ping(frame.payload))
            }
            WebSocketOpcode::Pong => {
                Ok(WebSocketMessage::Pong(frame.payload))
            }
            WebSocketOpcode::Close => {
                if frame.payload.is_empty() {
                    Ok(WebSocketMessage::Close(None))
                } else if frame.payload.len() < 2 {
                    return Err(WebSocketError::ProtocolViolation(
                        "Close frame with payload must have at least 2 bytes".to_string()
                    ));
                } else {
                    let code_bytes = [frame.payload[0], frame.payload[1]];
                    let code = WebSocketCloseCode::from_u16(u16::from_be_bytes(code_bytes));
                    let reason = if frame.payload.len() > 2 {
                        String::from_utf8(frame.payload[2..].to_vec())
                            .map_err(|e| WebSocketError::InvalidUtf8(e))?
                    } else {
                        String::new()
                    };
                    Ok(WebSocketMessage::Close(Some(WebSocketCloseFrame { code, reason })))
                }
            }
            WebSocketOpcode::Continuation => {
                // For simplicity, treat continuation frames as binary for now
                // A full implementation would need to handle frame fragmentation
                Ok(WebSocketMessage::Binary(frame.payload))
            }
        }
    }
    
    /// Convert WebSocket message to frame.
    fn message_to_frame(&self, message: WebSocketMessage) -> WebSocketFrame {
        let mask = if self.is_server {
            None // Server frames are not masked
        } else {
            // Client frames must be masked with random key
            let mut rng = rand::thread_rng();
            let bytes: [u8; 4] = rng.r#gen();
            Some(bytes)
        };
        
        match message {
            WebSocketMessage::Text(text) => {
                WebSocketFrame::text(text, mask)
            }
            WebSocketMessage::Binary(data) => {
                WebSocketFrame::binary(data, mask)
            }
            WebSocketMessage::Ping(data) => {
                WebSocketFrame::ping(data, mask)
            }
            WebSocketMessage::Pong(data) => {
                WebSocketFrame::pong(data, mask)
            }
            WebSocketMessage::Close(close_frame) => {
                if let Some(frame) = close_frame {
                    WebSocketFrame::close(Some(frame.code), Some(frame.reason), mask)
                } else {
                    WebSocketFrame::close(None, None, mask)
                }
            }
        }
    }
    
    /// Encode a WebSocket frame to bytes.
    fn encode_frame(&self, frame: WebSocketFrame, dst: &mut BytesMut) -> Result<(), WebSocketError> {
        let mut first_byte = frame.opcode as u8;
        if frame.fin {
            first_byte |= 0x80;
        }
        if frame.rsv1 {
            first_byte |= 0x40;
        }
        if frame.rsv2 {
            first_byte |= 0x20;
        }
        if frame.rsv3 {
            first_byte |= 0x10;
        }
        
        dst.put_u8(first_byte);
        
        let payload_len = frame.payload.len();
        let mut second_byte = 0u8;
        
        if frame.mask.is_some() {
            second_byte |= 0x80;
        }
        
        if payload_len < 126 {
            second_byte |= payload_len as u8;
            dst.put_u8(second_byte);
        } else if payload_len < 65536 {
            second_byte |= 126;
            dst.put_u8(second_byte);
            dst.put_u16(payload_len as u16);
        } else {
            second_byte |= 127;
            dst.put_u8(second_byte);
            dst.put_u64(payload_len as u64);
        }
        
        // Write masking key if present
        if let Some(mask) = frame.mask {
            dst.put_slice(&mask);
        }
        
        // Write payload (apply masking if needed)
        if let Some(mask) = frame.mask {
            for (i, &byte) in frame.payload.iter().enumerate() {
                dst.put_u8(byte ^ mask[i % 4]);
            }
        } else {
            dst.put_slice(&frame.payload);
        }
        
        Ok(())
    }
}

impl Decoder for WebSocketCodec {
    type Item = WebSocketMessage;
    type Error = WebSocketError;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                CodecState::Header => {
                    if let Some(header) = self.parse_header(src)? {
                        if header.payload_len == 0 {
                            // Empty payload - create frame immediately
                            let frame = WebSocketFrame {
                                fin: header.fin,
                                rsv1: header.rsv1,
                                rsv2: header.rsv2,
                                rsv3: header.rsv3,
                                opcode: header.opcode,
                                mask: header.mask,
                                payload: Vec::new(),
                            };
                            
                            return Ok(Some(Self::frame_to_message(frame)?));
                        } else {
                            // Need to read payload
                            self.state = CodecState::Payload {
                                header,
                                bytes_remaining: header.payload_len as usize,
                            };
                        }
                    } else {
                        return Ok(None); // Need more data
                    }
                }
                CodecState::Payload { header, bytes_remaining } => {
                    if src.len() < bytes_remaining {
                        return Ok(None); // Need more data
                    }
                    
                    // Read payload
                    let mut payload = vec![0u8; bytes_remaining];
                    src.copy_to_slice(&mut payload);
                    
                    // Apply unmasking if needed
                    if let Some(mask) = header.mask {
                        for (i, byte) in payload.iter_mut().enumerate() {
                            *byte ^= mask[i % 4];
                        }
                    }
                    
                    let frame = WebSocketFrame {
                        fin: header.fin,
                        rsv1: header.rsv1,
                        rsv2: header.rsv2,
                        rsv3: header.rsv3,
                        opcode: header.opcode,
                        mask: header.mask,
                        payload,
                    };
                    
                    self.state = CodecState::Header;
                    return Ok(Some(Self::frame_to_message(frame)?));
                }
            }
        }
    }
}

impl Encoder<WebSocketMessage> for WebSocketCodec {
    type Error = WebSocketError;
    
    fn encode(&mut self, message: WebSocketMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let frame = self.message_to_frame(message);
        self.encode_frame(frame, dst)
    }
}

impl Default for WebSocketServerCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketServerCodec {
    /// Create a new WebSocket server codec.
    pub fn new() -> Self {
        Self {
            http_codec: HttpCodec::new(),
            ws_codec: None,
        }
    }
    
    /// Check if the codec has been upgraded to WebSocket.
    pub fn is_upgraded(&self) -> bool {
        self.ws_codec.is_some()
    }
    
    /// Upgrade the codec to WebSocket mode.
    pub fn upgrade(&mut self) {
        self.ws_codec = Some(WebSocketCodec::server());
    }
}

impl Default for WebSocketClientCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketClientCodec {
    /// Create a new WebSocket client codec.
    pub fn new() -> Self {
        Self {
            http_codec: HttpCodec::new(),
            ws_codec: None,
        }
    }
    
    /// Check if the codec has been upgraded to WebSocket.
    pub fn is_upgraded(&self) -> bool {
        self.ws_codec.is_some()
    }
    
    /// Upgrade the codec to WebSocket mode.
    pub fn upgrade(&mut self) {
        self.ws_codec = Some(WebSocketCodec::client());
    }
}

// For now, we'll implement simplified server/client codecs that only handle WebSocket frames
// The full HTTP upgrade handshake will be implemented in the handshake module

impl Decoder for WebSocketServerCodec {
    type Item = WebSocketMessage;
    type Error = WebSocketError;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(ref mut ws_codec) = self.ws_codec {
            ws_codec.decode(src)
        } else {
            // For simplicity, auto-upgrade for now
            // In a full implementation, this would handle the HTTP handshake first
            self.upgrade();
            Ok(None)
        }
    }
}

impl Encoder<WebSocketMessage> for WebSocketServerCodec {
    type Error = WebSocketError;
    
    fn encode(&mut self, message: WebSocketMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if let Some(ref mut ws_codec) = self.ws_codec {
            ws_codec.encode(message, dst)
        } else {
            Err(WebSocketError::ProtocolViolation(
                "Cannot send WebSocket message before upgrade".to_string()
            ))
        }
    }
}

impl Decoder for WebSocketClientCodec {
    type Item = WebSocketMessage;
    type Error = WebSocketError;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(ref mut ws_codec) = self.ws_codec {
            ws_codec.decode(src)
        } else {
            // For simplicity, auto-upgrade for now
            // In a full implementation, this would handle the HTTP handshake first
            self.upgrade();
            Ok(None)
        }
    }
}

impl Encoder<WebSocketMessage> for WebSocketClientCodec {
    type Error = WebSocketError;
    
    fn encode(&mut self, message: WebSocketMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if let Some(ref mut ws_codec) = self.ws_codec {
            ws_codec.encode(message, dst)
        } else {
            Err(WebSocketError::ProtocolViolation(
                "Cannot send WebSocket message before upgrade".to_string()
            ))
        }
    }
}

// Convert WebSocketError to io::Error for compatibility with tokio-util codec traits
impl From<WebSocketError> for io::Error {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::Io(io_err) => io_err,
            other => io::Error::new(io::ErrorKind::Other, format!("{}", other)),
        }
    }
}
