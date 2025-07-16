//! HTTP types and core data structures.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// HTTP request structure compatible with DFIR streaming.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path (without query parameters)
    pub path: String,
    /// HTTP version (typically "HTTP/1.1")
    pub version: String,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Vec<u8>,
    /// Parsed query parameters
    pub query_params: HashMap<String, String>,
    /// Parsed cookies from Cookie header
    pub cookies: HashMap<String, String>,
    /// Parsed form data (for application/x-www-form-urlencoded)
    pub form_data: HashMap<String, String>,
}

/// HTTP response structure compatible with DFIR streaming.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP version (typically "HTTP/1.1")
    pub version: String,
    /// HTTP status code (200, 404, etc.)
    pub status_code: u16,
    /// HTTP status text ("OK", "Not Found", etc.)
    pub status_text: String,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Cookies to be set in the response
    pub set_cookies: Vec<Cookie>,
}

/// Represents an HTTP cookie with all its attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain attribute
    pub domain: Option<String>,
    /// Path attribute
    pub path: Option<String>,
    /// Max-Age attribute (in seconds)
    pub max_age: Option<i64>,
    /// Expires attribute
    pub expires: Option<String>,
    /// Secure flag
    pub secure: bool,
    /// HttpOnly flag
    pub http_only: bool,
    /// SameSite attribute
    pub same_site: Option<SameSite>,
}

/// SameSite cookie attribute values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SameSite {
    /// Strict SameSite policy
    Strict,
    /// Lax SameSite policy
    Lax,
    /// None SameSite policy (requires Secure)
    None,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

/// HTTP codec errors.
#[derive(Debug, Clone)]
pub enum HttpCodecError {
    /// Invalid HTTP request format
    InvalidRequest,
    /// Invalid HTTP response format
    InvalidResponse,
    /// Invalid Content-Length header
    InvalidContentLength,
    /// HTTP headers too large
    HeadersTooLarge,
    /// HTTP body too large
    BodyTooLarge,
    /// I/O error
    Io(String),
}

impl fmt::Display for HttpCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpCodecError::InvalidRequest => write!(f, "Invalid HTTP request format"),
            HttpCodecError::InvalidResponse => write!(f, "Invalid HTTP response format"),
            HttpCodecError::InvalidContentLength => write!(f, "Invalid Content-Length header"),
            HttpCodecError::HeadersTooLarge => write!(f, "HTTP headers too large"),
            HttpCodecError::BodyTooLarge => write!(f, "HTTP body too large"),
            HttpCodecError::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for HttpCodecError {}

impl From<std::io::Error> for HttpCodecError {
    fn from(err: std::io::Error) -> Self {
        HttpCodecError::Io(err.to_string())
    }
}
