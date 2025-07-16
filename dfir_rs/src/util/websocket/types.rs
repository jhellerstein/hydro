//! WebSocket message types and frame structures.

use std::fmt;
use serde::{Deserialize, Serialize};

/// WebSocket message types for application-level communication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebSocketMessage {
    /// Text message containing UTF-8 encoded string data.
    Text(String),
    
    /// Binary message containing raw byte data.
    Binary(Vec<u8>),
    
    /// Ping frame with optional payload (used for keep-alive).
    Ping(Vec<u8>),
    
    /// Pong frame with optional payload (response to ping).
    Pong(Vec<u8>),
    
    /// Close frame with optional status code and reason.
    Close(Option<WebSocketCloseFrame>),
}

/// WebSocket close frame data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSocketCloseFrame {
    /// Close status code.
    pub code: WebSocketCloseCode,
    /// Optional reason for closing.
    pub reason: String,
}

/// WebSocket close status codes as defined in RFC 6455.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum WebSocketCloseCode {
    /// Normal closure; the connection successfully completed the purpose for which it was created.
    Normal = 1000,
    
    /// The endpoint is going away, either because of a server failure or because the browser is navigating away.
    GoingAway = 1001,
    
    /// The endpoint is terminating the connection due to a protocol error.
    ProtocolError = 1002,
    
    /// The connection is being terminated because the endpoint received data of a type it cannot accept.
    UnsupportedData = 1003,
    
    /// Reserved. A meaning might be defined in the future.
    Reserved = 1004,
    
    /// Reserved. Must not be set as a status code in a Close control frame by an endpoint.
    NoStatusReceived = 1005,
    
    /// Reserved. Must not be set as a status code in a Close control frame by an endpoint.
    AbnormalClosure = 1006,
    
    /// The endpoint is terminating the connection because a message was received that contained inconsistent data.
    InvalidFramePayloadData = 1007,
    
    /// The endpoint is terminating the connection because it received a message that violates its policy.
    PolicyViolation = 1008,
    
    /// The endpoint is terminating the connection because a data frame was received that is too large.
    MessageTooBig = 1009,
    
    /// The client is terminating the connection because it expected the server to negotiate one or more extension.
    MandatoryExtension = 1010,
    
    /// The server is terminating the connection because it encountered an unexpected condition.
    InternalError = 1011,
    
    /// The connection is being closed due to a failure to perform a TLS handshake.
    ServiceRestart = 1012,
    
    /// The connection is being closed due to a failure to perform a TLS handshake.
    TryAgainLater = 1013,
    
    /// The server was acting as a gateway or proxy and received an invalid response from the upstream server.
    BadGateway = 1014,
    
    /// Reserved. Must not be set as a status code in a Close control frame by an endpoint.
    TlsHandshake = 1015,
}

/// WebSocket frame opcodes as defined in RFC 6455.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WebSocketOpcode {
    /// Continuation frame.
    Continuation = 0x0,
    
    /// Text frame.
    Text = 0x1,
    
    /// Binary frame.
    Binary = 0x2,
    
    /// Connection close frame.
    Close = 0x8,
    
    /// Ping frame.
    Ping = 0x9,
    
    /// Pong frame.
    Pong = 0xA,
}

/// Low-level WebSocket frame structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketFrame {
    /// Final fragment flag.
    pub fin: bool,
    
    /// Reserved bits (must be 0).
    pub rsv1: bool,
    /// Reserved bit 2 (must be 0).
    pub rsv2: bool,
    /// Reserved bit 3 (must be 0).
    pub rsv3: bool,
    
    /// Frame opcode.
    pub opcode: WebSocketOpcode,
    
    /// Masking key for client frames (None for server frames).
    pub mask: Option<[u8; 4]>,
    
    /// Frame payload data.
    pub payload: Vec<u8>,
}

/// WebSocket protocol errors.
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    /// Invalid frame format.
    #[error("Invalid frame format: {0}")]
    InvalidFrame(String),
    
    /// Protocol violation.
    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),
    
    /// Invalid UTF-8 in text frame.
    #[error("Invalid UTF-8 in text frame: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    
    /// Frame too large.
    #[error("Frame too large: {size} bytes (max: {max_size})")]
    FrameTooLarge { 
        /// The actual size of the frame
        size: usize, 
        /// The maximum allowed size
        max_size: usize 
    },
    
    /// Connection closed.
    #[error("Connection closed: {code:?} - {reason}")]
    ConnectionClosed {
        /// The close code sent by the peer
        code: Option<WebSocketCloseCode>,
        /// The close reason sent by the peer
        reason: String,
    },
    
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// HTTP handshake error.
    #[error("HTTP handshake error: {0}")]
    HandshakeError(String),
}

impl WebSocketMessage {
    /// Create a new text message.
    pub fn text<S: Into<String>>(text: S) -> Self {
        WebSocketMessage::Text(text.into())
    }
    
    /// Create a new binary message.
    pub fn binary<B: Into<Vec<u8>>>(data: B) -> Self {
        WebSocketMessage::Binary(data.into())
    }
    
    /// Create a new ping message.
    pub fn ping<B: Into<Vec<u8>>>(data: B) -> Self {
        WebSocketMessage::Ping(data.into())
    }
    
    /// Create a new pong message.
    pub fn pong<B: Into<Vec<u8>>>(data: B) -> Self {
        WebSocketMessage::Pong(data.into())
    }
    
    /// Create a new close message.
    pub fn close(code: WebSocketCloseCode, reason: String) -> Self {
        WebSocketMessage::Close(Some(WebSocketCloseFrame { code, reason }))
    }
    
    /// Create a close message without reason.
    pub fn close_empty() -> Self {
        WebSocketMessage::Close(None)
    }
    
    /// Returns true if this is a text message.
    pub fn is_text(&self) -> bool {
        matches!(self, WebSocketMessage::Text(_))
    }
    
    /// Returns true if this is a binary message.
    pub fn is_binary(&self) -> bool {
        matches!(self, WebSocketMessage::Binary(_))
    }
    
    /// Returns true if this is a ping message.
    pub fn is_ping(&self) -> bool {
        matches!(self, WebSocketMessage::Ping(_))
    }
    
    /// Returns true if this is a pong message.
    pub fn is_pong(&self) -> bool {
        matches!(self, WebSocketMessage::Pong(_))
    }
    
    /// Returns true if this is a close message.
    pub fn is_close(&self) -> bool {
        matches!(self, WebSocketMessage::Close(_))
    }
    
    /// Get the text content if this is a text message.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            WebSocketMessage::Text(text) => Some(text),
            _ => None,
        }
    }
    
    /// Get the binary content if this is a binary message.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            WebSocketMessage::Binary(data) => Some(data),
            _ => None,
        }
    }
}

impl WebSocketOpcode {
    /// Create opcode from u8 value.
    pub fn from_u8(value: u8) -> Result<Self, WebSocketError> {
        match value {
            0x0 => Ok(WebSocketOpcode::Continuation),
            0x1 => Ok(WebSocketOpcode::Text),
            0x2 => Ok(WebSocketOpcode::Binary),
            0x8 => Ok(WebSocketOpcode::Close),
            0x9 => Ok(WebSocketOpcode::Ping),
            0xA => Ok(WebSocketOpcode::Pong),
            _ => Err(WebSocketError::InvalidFrame(format!("Unknown opcode: 0x{:02X}", value))),
        }
    }
    
    /// Returns true if this is a control frame opcode.
    pub fn is_control(self) -> bool {
        matches!(self, WebSocketOpcode::Close | WebSocketOpcode::Ping | WebSocketOpcode::Pong)
    }
    
    /// Returns true if this is a data frame opcode.
    pub fn is_data(self) -> bool {
        matches!(self, WebSocketOpcode::Text | WebSocketOpcode::Binary | WebSocketOpcode::Continuation)
    }
}

impl WebSocketCloseCode {
    /// Create close code from u16 value.
    pub fn from_u16(value: u16) -> Self {
        match value {
            1000 => WebSocketCloseCode::Normal,
            1001 => WebSocketCloseCode::GoingAway,
            1002 => WebSocketCloseCode::ProtocolError,
            1003 => WebSocketCloseCode::UnsupportedData,
            1004 => WebSocketCloseCode::Reserved,
            1005 => WebSocketCloseCode::NoStatusReceived,
            1006 => WebSocketCloseCode::AbnormalClosure,
            1007 => WebSocketCloseCode::InvalidFramePayloadData,
            1008 => WebSocketCloseCode::PolicyViolation,
            1009 => WebSocketCloseCode::MessageTooBig,
            1010 => WebSocketCloseCode::MandatoryExtension,
            1011 => WebSocketCloseCode::InternalError,
            1012 => WebSocketCloseCode::ServiceRestart,
            1013 => WebSocketCloseCode::TryAgainLater,
            1014 => WebSocketCloseCode::BadGateway,
            1015 => WebSocketCloseCode::TlsHandshake,
            _ => WebSocketCloseCode::InternalError, // Default for unknown codes
        }
    }
    
    /// Get the numeric value of the close code.
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

impl WebSocketFrame {
    /// Create a new WebSocket frame.
    pub fn new(fin: bool, opcode: WebSocketOpcode, mask: Option<[u8; 4]>, payload: Vec<u8>) -> Self {
        WebSocketFrame {
            fin,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode,
            mask,
            payload,
        }
    }
    
    /// Create a text frame.
    pub fn text<S: AsRef<str>>(text: S, mask: Option<[u8; 4]>) -> Self {
        WebSocketFrame::new(true, WebSocketOpcode::Text, mask, text.as_ref().as_bytes().to_vec())
    }
    
    /// Create a binary frame.
    pub fn binary<B: Into<Vec<u8>>>(data: B, mask: Option<[u8; 4]>) -> Self {
        WebSocketFrame::new(true, WebSocketOpcode::Binary, mask, data.into())
    }
    
    /// Create a ping frame.
    pub fn ping<B: Into<Vec<u8>>>(data: B, mask: Option<[u8; 4]>) -> Self {
        WebSocketFrame::new(true, WebSocketOpcode::Ping, mask, data.into())
    }
    
    /// Create a pong frame.
    pub fn pong<B: Into<Vec<u8>>>(data: B, mask: Option<[u8; 4]>) -> Self {
        WebSocketFrame::new(true, WebSocketOpcode::Pong, mask, data.into())
    }
    
    /// Create a close frame.
    pub fn close(code: Option<WebSocketCloseCode>, reason: Option<String>, mask: Option<[u8; 4]>) -> Self {
        let mut payload = Vec::new();
        
        if let Some(code) = code {
            payload.extend_from_slice(&code.as_u16().to_be_bytes());
            if let Some(reason) = reason {
                payload.extend_from_slice(reason.as_bytes());
            }
        }
        
        WebSocketFrame::new(true, WebSocketOpcode::Close, mask, payload)
    }
    
    /// Apply or remove masking to the payload.
    pub fn apply_mask(&mut self) {
        if let Some(mask) = self.mask {
            for (i, byte) in self.payload.iter_mut().enumerate() {
                *byte ^= mask[i % 4];
            }
        }
    }
    
    /// Check if this frame is masked.
    pub fn is_masked(&self) -> bool {
        self.mask.is_some()
    }
    
    /// Check if this is a control frame.
    pub fn is_control(&self) -> bool {
        self.opcode.is_control()
    }
    
    /// Check if this is a data frame.
    pub fn is_data(&self) -> bool {
        self.opcode.is_data()
    }
}

impl fmt::Display for WebSocketMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketMessage::Text(text) => write!(f, "Text({})", text),
            WebSocketMessage::Binary(data) => write!(f, "Binary({} bytes)", data.len()),
            WebSocketMessage::Ping(data) => write!(f, "Ping({} bytes)", data.len()),
            WebSocketMessage::Pong(data) => write!(f, "Pong({} bytes)", data.len()),
            WebSocketMessage::Close(close_frame) => {
                if let Some(frame) = close_frame {
                    write!(f, "Close({:?}: {})", frame.code, frame.reason)
                } else {
                    write!(f, "Close")
                }
            }
        }
    }
}

impl fmt::Display for WebSocketCloseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            WebSocketCloseCode::Normal => "Normal",
            WebSocketCloseCode::GoingAway => "Going Away",
            WebSocketCloseCode::ProtocolError => "Protocol Error",
            WebSocketCloseCode::UnsupportedData => "Unsupported Data",
            WebSocketCloseCode::Reserved => "Reserved",
            WebSocketCloseCode::NoStatusReceived => "No Status Received",
            WebSocketCloseCode::AbnormalClosure => "Abnormal Closure",
            WebSocketCloseCode::InvalidFramePayloadData => "Invalid Frame Payload Data",
            WebSocketCloseCode::PolicyViolation => "Policy Violation",
            WebSocketCloseCode::MessageTooBig => "Message Too Big",
            WebSocketCloseCode::MandatoryExtension => "Mandatory Extension",
            WebSocketCloseCode::InternalError => "Internal Error",
            WebSocketCloseCode::ServiceRestart => "Service Restart",
            WebSocketCloseCode::TryAgainLater => "Try Again Later",
            WebSocketCloseCode::BadGateway => "Bad Gateway",
            WebSocketCloseCode::TlsHandshake => "TLS Handshake",
        };
        write!(f, "{} ({})", name, self.as_u16())
    }
}
