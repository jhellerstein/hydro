//! WebSocket connection management.

use super::types::{WebSocketMessage, WebSocketError, WebSocketCloseCode};

/// WebSocket connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established.
    Connecting,
    /// Connection is established and ready for communication.
    Open,
    /// Connection is being closed.
    Closing,
    /// Connection is closed.
    Closed,
}

/// WebSocket connection manager.
#[derive(Debug)]
pub struct WebSocketConnection {
    /// Current connection state.
    state: ConnectionState,
}

impl Default for WebSocketConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketConnection {
    /// Create a new WebSocket connection.
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Connecting,
        }
    }
    
    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        self.state
    }
    
    /// Mark the connection as open.
    pub fn open(&mut self) {
        self.state = ConnectionState::Open;
    }
    
    /// Mark the connection as closing.
    pub fn close(&mut self) {
        self.state = ConnectionState::Closing;
    }
    
    /// Mark the connection as closed.
    pub fn closed(&mut self) {
        self.state = ConnectionState::Closed;
    }
    
    /// Check if the connection is open.
    pub fn is_open(&self) -> bool {
        self.state == ConnectionState::Open
    }
    
    /// Check if the connection is closed.
    pub fn is_closed(&self) -> bool {
        self.state == ConnectionState::Closed
    }
    
    /// Handle an incoming message and update connection state.
    pub fn handle_message(&mut self, message: &WebSocketMessage) -> Result<Option<WebSocketMessage>, WebSocketError> {
        match message {
            WebSocketMessage::Close(_) => {
                if self.state == ConnectionState::Open {
                    self.close();
                    // Respond with close frame
                    Ok(Some(WebSocketMessage::close(WebSocketCloseCode::Normal, "".to_string())))
                } else {
                    self.closed();
                    Ok(None)
                }
            }
            WebSocketMessage::Ping(data) => {
                // Respond to ping with pong
                Ok(Some(WebSocketMessage::pong(data.clone())))
            }
            _ => {
                // Regular message, no special handling needed
                Ok(None)
            }
        }
    }
}
