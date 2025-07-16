//! WebSocket handshake implementation.
//!
//! This module handles the HTTP upgrade handshake required to establish WebSocket connections.
//! It follows RFC 6455 Section 4 for the opening handshake.

use crate::util::http::{HttpRequest, HttpResponse};
use super::types::WebSocketError;
use sha1::{Digest, Sha1};
use base64::Engine;
use rand::Rng;
use std::collections::HashMap;

/// WebSocket GUID as defined in RFC 6455.
const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// WebSocket handshake request.
#[derive(Debug, Clone)]
pub struct HandshakeRequest {
    /// The HTTP request for the WebSocket upgrade.
    pub request: HttpRequest,
    /// WebSocket key for the handshake.
    pub key: String,
    /// WebSocket version.
    pub version: String,
    /// Optional subprotocols.
    pub protocols: Vec<String>,
}

/// WebSocket handshake response.
#[derive(Debug, Clone)]
pub struct HandshakeResponse {
    /// The HTTP response for the WebSocket upgrade.
    pub response: HttpResponse,
    /// WebSocket accept key.
    pub accept: String,
    /// Selected subprotocol.
    pub protocol: Option<String>,
}

/// WebSocket handshake utilities.
pub struct WebSocketHandshake;

impl WebSocketHandshake {
    /// Generate a WebSocket key for client handshake.
    /// 
    /// The key is a base64-encoded 16-byte random value as per RFC 6455.
    pub fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        let key_bytes: [u8; 16] = rng.r#gen();
        base64::engine::general_purpose::STANDARD.encode(key_bytes)
    }

    /// Calculate the WebSocket accept key from the client key.
    /// 
    /// The accept key is calculated by concatenating the client key with the WebSocket GUID,
    /// then taking the SHA-1 hash and base64-encoding the result.
    pub fn calculate_accept_key(key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WEBSOCKET_GUID.as_bytes());
        let hash = hasher.finalize();
        base64::engine::general_purpose::STANDARD.encode(hash)
    }

    /// Create a WebSocket handshake request.
    pub fn create_request(uri: &str, protocols: Vec<String>) -> Result<HandshakeRequest, WebSocketError> {
        let key = Self::generate_key();
        let mut headers = HashMap::new();
        
        // Required WebSocket headers
        headers.insert("Host".to_string(), uri.to_string());
        headers.insert("Upgrade".to_string(), "websocket".to_string());
        headers.insert("Connection".to_string(), "Upgrade".to_string());
        headers.insert("Sec-WebSocket-Key".to_string(), key.clone());
        headers.insert("Sec-WebSocket-Version".to_string(), "13".to_string());
        
        // Optional protocol header
        if !protocols.is_empty() {
            headers.insert("Sec-WebSocket-Protocol".to_string(), protocols.join(", "));
        }

        let http_request = HttpRequest {
            method: "GET".to_string(),
            path: uri.to_string(),
            version: "HTTP/1.1".to_string(),
            headers,
            body: Vec::new(),
            query_params: HashMap::new(),
            cookies: HashMap::new(),
            form_data: HashMap::new(),
        };

        Ok(HandshakeRequest {
            request: http_request,
            key,
            version: "13".to_string(),
            protocols,
        })
    }
    
    /// Create a WebSocket handshake response.
    pub fn create_response(request: &HandshakeRequest) -> Result<HandshakeResponse, WebSocketError> {
        // Validate the handshake request first
        Self::validate_handshake_headers(&request.request)?;
        
        let accept_key = Self::calculate_accept_key(&request.key);
        let mut headers = HashMap::new();
        
        // Required WebSocket response headers
        headers.insert("Upgrade".to_string(), "websocket".to_string());
        headers.insert("Connection".to_string(), "Upgrade".to_string());
        headers.insert("Sec-WebSocket-Accept".to_string(), accept_key.clone());

        // Handle subprotocol selection
        let selected_protocol = if !request.protocols.is_empty() {
            // For now, just select the first protocol
            // In a real implementation, you'd have logic to choose the best protocol
            let protocol = request.protocols[0].clone();
            headers.insert("Sec-WebSocket-Protocol".to_string(), protocol.clone());
            Some(protocol)
        } else {
            None
        };

        let http_response = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: 101, // HTTP 101 Switching Protocols
            status_text: "Switching Protocols".to_string(),
            headers,
            body: Vec::new(),
            set_cookies: Vec::new(),
        };

        Ok(HandshakeResponse {
            response: http_response,
            accept: accept_key,
            protocol: selected_protocol,
        })
    }
    
    /// Validate a WebSocket handshake request.
    pub fn validate_request(request: &HttpRequest) -> Result<HandshakeRequest, WebSocketError> {
        // Check method
        if request.method != "GET" {
            return Err(WebSocketError::HandshakeError(
                "WebSocket handshake must use GET method".to_string()
            ));
        }

        // Validate required headers
        Self::validate_handshake_headers(request)?;

        // Extract WebSocket key
        let key = request.headers.get("sec-websocket-key")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Sec-WebSocket-Key header".to_string()
            ))?
            .clone();

        // Extract version
        let version = request.headers.get("sec-websocket-version")
            .unwrap_or(&"13".to_string())
            .clone();

        // Extract protocols
        let protocols = request.headers.get("sec-websocket-protocol")
            .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        Ok(HandshakeRequest {
            request: request.clone(),
            key,
            version,
            protocols,
        })
    }
    
    /// Validate a WebSocket handshake response.
    pub fn validate_response(response: &HttpResponse, expected_key: &str) -> Result<HandshakeResponse, WebSocketError> {
        // Check status code
        if response.status_code != 101 {
            return Err(WebSocketError::HandshakeError(
                format!("Expected status 101, got {}", response.status_code)
            ));
        }

        // Check upgrade header
        let upgrade = response.headers.get("upgrade")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Upgrade header in response".to_string()
            ))?;
        
        if upgrade.to_lowercase() != "websocket" {
            return Err(WebSocketError::HandshakeError(
                "Upgrade header must be 'websocket'".to_string()
            ));
        }

        // Check connection header
        let connection = response.headers.get("connection")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Connection header in response".to_string()
            ))?;
        
        if !connection.to_lowercase().contains("upgrade") {
            return Err(WebSocketError::HandshakeError(
                "Connection header must contain 'upgrade'".to_string()
            ));
        }

        // Validate accept key
        let accept = response.headers.get("sec-websocket-accept")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Sec-WebSocket-Accept header".to_string()
            ))?;
        
        let expected_accept = Self::calculate_accept_key(expected_key);
        if accept != &expected_accept {
            return Err(WebSocketError::HandshakeError(
                "Invalid Sec-WebSocket-Accept key".to_string()
            ));
        }

        // Extract selected protocol
        let protocol = response.headers.get("sec-websocket-protocol")
            .map(|p| p.clone());

        Ok(HandshakeResponse {
            response: response.clone(),
            accept: accept.clone(),
            protocol,
        })
    }

    /// Validate required WebSocket handshake headers.
    fn validate_handshake_headers(request: &HttpRequest) -> Result<(), WebSocketError> {
        // Check upgrade header
        let upgrade = request.headers.get("upgrade")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Upgrade header".to_string()
            ))?;
        
        if upgrade.to_lowercase() != "websocket" {
            return Err(WebSocketError::HandshakeError(
                "Upgrade header must be 'websocket'".to_string()
            ));
        }

        // Check connection header
        let connection = request.headers.get("connection")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Connection header".to_string()
            ))?;
        
        if !connection.to_lowercase().contains("upgrade") {
            return Err(WebSocketError::HandshakeError(
                "Connection header must contain 'upgrade'".to_string()
            ));
        }

        // Check WebSocket version
        let version = request.headers.get("sec-websocket-version")
            .map(|s| s.as_str())
            .unwrap_or("13");
        
        if version != "13" {
            return Err(WebSocketError::HandshakeError(
                format!("Unsupported WebSocket version: {}", version)
            ));
        }

        // Check WebSocket key presence
        let key = request.headers.get("sec-websocket-key")
            .ok_or_else(|| WebSocketError::HandshakeError(
                "Missing Sec-WebSocket-Key header".to_string()
            ))?;

        // Key should be 24 characters when base64-encoded (16 bytes -> 24 chars)
        if key.len() != 24 {
            return Err(WebSocketError::HandshakeError(
                "Invalid Sec-WebSocket-Key length".to_string()
            ));
        }

        Ok(())
    }
}
