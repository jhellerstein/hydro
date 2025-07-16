#![cfg(not(target_arch = "wasm32"))]

//! HTTP support for DFIR.
//!
//! This module provides HTTP request and response types along with codecs for parsing
//! HTTP messages over TCP connections. It's designed to work seamlessly with DFIR's
//! stream processing model.
//!
//! ## Features
//!
//! - **HTTP/1.1 compliance**: Supports both Content-Length and Transfer-Encoding: chunked
//! - **Multiple HTTP methods**: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS with convenience builders
//! - **Query parameters**: Automatic parsing and URL encoding/decoding
//! - **Cookie support**: Request cookie parsing and response Set-Cookie headers with full attribute support
//! - **JSON support**: Easy JSON request/response handling with automatic headers
//! - **Status code helpers**: Comprehensive set of status code constructors (2xx, 3xx, 4xx, 5xx)
//! - **Error handling**: Comprehensive error types for different failure modes
//! - **Streaming**: Works with DFIR's streaming model for real-time HTTP processing
//!
//! ## Cookie Support
//!
//! The HTTP module provides comprehensive cookie support:
//!
//! - **Request cookies**: Automatically parsed from `Cookie` headers into `request.cookies` HashMap
//! - **Response cookies**: Set via `with_cookie()` or `with_simple_cookie()` methods
//! - **Cookie attributes**: Domain, Path, Max-Age, Expires, Secure, HttpOnly, SameSite
//! - **Authentication patterns**: Cookie-based session management and user preferences
//!
//! ```rust,no_run
//! use dfir_rs::util::{Cookie, HttpRequest, HttpResponse, SameSite};
//!
//! // Reading cookies from requests
//! let request = HttpRequest::get("/dashboard");
//! if let Some(session_id) = request.get_cookie("session_id") {
//!     println!("User session: {}", session_id);
//! }
//!
//! // PATCH for partial updates
//! let user_update = serde_json::json!({
//!     "email": "new@example.com",
//!     "preferences": {"theme": "dark"}
//! });
//! let patch_req = HttpRequest::patch_json("/api/users/123", &user_update).unwrap();
//!
//! // Setting cookies in responses
//! let response = HttpResponse::ok()
//!     .with_simple_cookie("session_id", "abc123")
//!     .with_cookie(
//!         Cookie::new("auth_token", "xyz789")
//!             .with_domain("example.com")
//!             .with_path("/app")
//!             .secure()
//!             .http_only()
//!             .with_same_site(SameSite::Strict),
//!     );
//! ```
//!
//! # Basic Usage
//!
//! ## HTTP Server
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
//!
//! use dfir_rs::util::{HttpResponse, bind_http_server};
//! use dfir_rs::{dfir_syntax, var_args};
//! use tokio::task::LocalSet;
//!
//! # async fn example() {
//! // HTTP operations require LocalSet for proper execution
//! let local = LocalSet::new();
//! let (response_send, request_recv, addr) = local
//!     .run_until(bind_http_server("127.0.0.1:8080".parse().unwrap()))
//!     .await;
//!
//! // Define your DFIR graph
//! let mut flow = dfir_syntax! {
//!     requests = source_stream(request_recv)
//!       -> map(|result| result.unwrap())  // Handle errors appropriately in real code
//!       -> demux(|(request, client_addr): (dfir_rs::util::HttpRequest, std::net::SocketAddr), var_args!(home, not_found)| {
//!           match request.path.as_str() {
//!               "/" => home.give((request, client_addr)),
//!               _ => not_found.give((request, client_addr)),
//!           }
//!       });
//!
//!     // Union responses before sending
//!     responses = union() -> dest_sink(response_send);
//!
//!     // Handle home page requests
//!     requests[home] -> map(|(_, client_addr)| {
//!         (HttpResponse::ok().with_body(b"Hello, World!".to_vec()), client_addr)
//!     }) -> responses;
//!
//!     // Handle 404s
//!     requests[not_found] -> map(|(_, client_addr)| {
//!         (HttpResponse::not_found(), client_addr)
//!     }) -> responses;
//! };
//!
//! // Run the flow
//! local.run_until(flow.run_async()).await;
//! # }
//! ```
//!
//! ## HTTP Client
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
//!
//! use dfir_rs::util::{HttpRequest, connect_http_client};
//! use dfir_rs::{dfir_syntax, var_args};
//! use tokio::task::LocalSet;
//!
//! # async fn example() {
//! // HTTP operations require LocalSet for proper execution
//! let local = LocalSet::new();
//! let (request_send, response_recv) = connect_http_client();
//!
//! // Define your DFIR graph
//! let mut flow = dfir_syntax! {
//!     source_iter([HttpRequest::get("/")])
//!       -> map(|req| (req, "127.0.0.1:8080".parse().unwrap()))
//!       -> dest_sink(request_send);
//!
//!     responses = source_stream(response_recv)
//!       -> demux(|result, var_args!(success, error)| {
//!           match result {
//!               Ok((response, addr)) => success.give((response, addr)),
//!               Err(e) => error.give(e),
//!           }
//!       });
//!
//!     // Handle successful responses
//!     responses[success] -> for_each(|(response, addr): (dfir_rs::util::HttpResponse, std::net::SocketAddr)| {
//!         println!("Got response: {} from {}", response.status_code, addr);
//!     });
//!
//!     // Handle errors
//!     responses[error] -> for_each(|e| {
//!         eprintln!("HTTP error: {}", e);
//!     });
//! };
//!
//! // Run the flow
//! local.run_until(flow.run_async()).await;
//! # }
//! ```

use std::collections::HashMap;
use std::fmt;

use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

/// Represents an HTTP cookie with its attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain attribute (optional)
    pub domain: Option<String>,
    /// Path attribute (optional)
    pub path: Option<String>,
    /// Max-Age attribute in seconds (optional)
    pub max_age: Option<i64>,
    /// Expires attribute (optional, as HTTP date string)
    pub expires: Option<String>,
    /// Secure flag
    pub secure: bool,
    /// HttpOnly flag
    pub http_only: bool,
    /// SameSite attribute
    pub same_site: Option<SameSite>,
}

/// SameSite cookie attribute values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSite {
    /// Strict SameSite policy
    Strict,
    /// Lax SameSite policy
    Lax,
    /// None SameSite policy (requires Secure)
    None,
}

/// A simple HTTP request representation for DFIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path (without query parameters)
    pub path: String,
    /// HTTP version (e.g., "HTTP/1.1")
    pub version: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Query parameters parsed from the URL
    pub query_params: HashMap<String, String>,
    /// Cookies parsed from the Cookie header
    pub cookies: HashMap<String, String>,
    /// Request body (empty for methods like GET)
    pub body: Vec<u8>,
}

/// A simple HTTP response representation for DFIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP version (e.g., "HTTP/1.1")
    pub version: String,
    /// Status code (e.g., 200)
    pub status_code: u16,
    /// Status text (e.g., "OK")
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Cookies to be set in the response
    pub set_cookies: Vec<Cookie>,
    /// Response body
    pub body: Vec<u8>,
}

impl Cookie {
    /// Create a new cookie with name and value.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            domain: None,
            path: None,
            max_age: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Set the domain attribute.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the path attribute.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the max-age attribute (in seconds).
    pub fn with_max_age(mut self, max_age: i64) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the expires attribute (HTTP date string).
    pub fn with_expires(mut self, expires: impl Into<String>) -> Self {
        self.expires = Some(expires.into());
        self
    }

    /// Mark the cookie as secure.
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Mark the cookie as HTTP-only.
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set the SameSite attribute.
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Parse cookies from a Cookie header value.
    /// Returns a HashMap of cookie name to value.
    pub fn parse_cookie_header(header_value: &str) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        for cookie_pair in header_value.split(';') {
            let cookie_pair = cookie_pair.trim();
            if let Some(eq_pos) = cookie_pair.find('=') {
                let (name, value) = cookie_pair.split_at(eq_pos);
                let value = &value[1..]; // Skip the '=' character

                let name = name.trim();
                let value = value.trim();

                if !name.is_empty() {
                    cookies.insert(name.to_string(), value.to_string());
                }
            }
        }

        cookies
    }

    /// Format this cookie for use in a Set-Cookie header.
    pub fn to_set_cookie_header(&self) -> String {
        let mut header = format!("{}={}", self.name, self.value);

        if let Some(ref domain) = self.domain {
            header.push_str(&format!("; Domain={}", domain));
        }

        if let Some(ref path) = self.path {
            header.push_str(&format!("; Path={}", path));
        }

        if let Some(max_age) = self.max_age {
            header.push_str(&format!("; Max-Age={}", max_age));
        }

        if let Some(ref expires) = self.expires {
            header.push_str(&format!("; Expires={}", expires));
        }

        if self.secure {
            header.push_str("; Secure");
        }

        if self.http_only {
            header.push_str("; HttpOnly");
        }

        if let Some(ref same_site) = self.same_site {
            let same_site_str = match same_site {
                SameSite::Strict => "Strict",
                SameSite::Lax => "Lax",
                SameSite::None => "None",
            };
            header.push_str(&format!("; SameSite={}", same_site_str));
        }

        header
    }
}

impl HttpRequest {
    /// Simple URL decoding for query parameters
    fn url_decode(encoded: &str) -> String {
        let mut decoded = String::new();
        let mut chars = encoded.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    // Try to decode hex sequence
                    if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                        if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                            let byte = (d1 as u8) << 4 | (d2 as u8);
                            if let Ok(utf8_char) = std::str::from_utf8(&[byte]) {
                                decoded.push_str(utf8_char);
                            } else {
                                // Invalid UTF-8, keep as-is
                                decoded.push('%');
                                decoded.push(h1);
                                decoded.push(h2);
                            }
                        } else {
                            // Invalid hex, keep as-is
                            decoded.push('%');
                            decoded.push(h1);
                            decoded.push(h2);
                        }
                    } else {
                        // Incomplete sequence, keep as-is
                        decoded.push('%');
                    }
                }
                '+' => decoded.push(' '), // + represents space in query strings
                _ => decoded.push(c),
            }
        }

        decoded
    }

    /// Parse query parameters from a URL path
    fn parse_query_params(path: &str) -> (String, HashMap<String, String>) {
        if let Some(query_start) = path.find('?') {
            let (path_part, query_part) = path.split_at(query_start);
            let query_string = &query_part[1..]; // Skip the '?' character

            let mut params = HashMap::new();
            for pair in query_string.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let (key, value) = pair.split_at(eq_pos);
                    let value = &value[1..]; // Skip the '=' character

                    // URL decode the key and value
                    let decoded_key = Self::url_decode(key);
                    let decoded_value = Self::url_decode(value);

                    params.insert(decoded_key, decoded_value);
                } else if !pair.is_empty() {
                    // Handle parameters without values (e.g., "?debug&verbose")
                    let decoded_key = Self::url_decode(pair);
                    params.insert(decoded_key, String::new());
                }
            }

            (path_part.to_string(), params)
        } else {
            (path.to_string(), HashMap::new())
        }
    }

    /// Create a simple GET request.
    pub fn get(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "GET".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params,
            cookies: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a simple POST request with JSON body.
    pub fn post_json(
        path: impl Into<String>,
        json: &impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(json)?;
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Ok(Self {
            method: "POST".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            query_params,
            cookies: HashMap::new(),
            body,
        })
    }

    /// Create a POST request with arbitrary body.
    pub fn post(path: impl Into<String>, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "POST".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            query_params,
            cookies: HashMap::new(),
            body,
        }
    }

    /// Create a PUT request.
    pub fn put(path: impl Into<String>, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "PUT".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            query_params,
            cookies: HashMap::new(),
            body,
        }
    }

    /// Create a PATCH request with arbitrary body.
    pub fn patch(path: impl Into<String>, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "PATCH".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            query_params,
            cookies: HashMap::new(),
            body,
        }
    }

    /// Create a PATCH request with JSON body.
    pub fn patch_json(
        path: impl Into<String>,
        json: &impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(json)?;
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Ok(Self {
            method: "PATCH".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            query_params,
            cookies: HashMap::new(),
            body,
        })
    }

    /// Create a DELETE request.
    pub fn delete(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "DELETE".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params,
            cookies: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a HEAD request.
    pub fn head(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "HEAD".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params,
            cookies: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create an OPTIONS request.
    pub fn options(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "OPTIONS".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params,
            cookies: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Add a header to the request.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Add a query parameter to the request.
    pub fn with_query_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(name.into(), value.into());
        self
    }

    /// Get a query parameter value by name.
    pub fn get_query_param(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)
    }

    /// Check if a query parameter exists (useful for flag-style parameters).
    pub fn has_query_param(&self, name: &str) -> bool {
        self.query_params.contains_key(name)
    }

    /// Get the full URL with query parameters.
    pub fn full_url(&self) -> String {
        if self.query_params.is_empty() {
            self.path.clone()
        } else {
            let query_string: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        Self::url_encode(k)
                    } else {
                        format!("{}={}", Self::url_encode(k), Self::url_encode(v))
                    }
                })
                .collect();
            format!("{}?{}", self.path, query_string.join("&"))
        }
    }

    /// Get a cookie value by name.
    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Check if a cookie exists.
    pub fn has_cookie(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }

    /// Simple URL encoding for query parameters
    fn url_encode(s: &str) -> String {
        let mut encoded = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => encoded.push('+'),
                _ => {
                    encoded.push('%');
                    encoded.push_str(&format!("{:02X}", byte));
                }
            }
        }
        encoded
    }
}

impl HttpResponse {
    /// Create a simple 200 OK response.
    pub fn ok() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a response with JSON body.
    pub fn json(json: &impl Serialize) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(json)?;
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());

        Ok(Self {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            status_text: "OK".to_string(),
            headers,
            set_cookies: Vec::new(),
            body,
        })
    }

    /// Create an error response.
    pub fn error(status_code: u16, status_text: impl Into<String>) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code,
            status_text: status_text.into(),
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a 404 Not Found response.
    pub fn not_found() -> Self {
        Self::error(404, "Not Found")
    }

    /// Create a 201 Created response.
    pub fn created() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 201,
            status_text: "Created".to_string(),
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a 204 No Content response.
    pub fn no_content() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 204,
            status_text: "No Content".to_string(),
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a 400 Bad Request response.
    pub fn bad_request() -> Self {
        Self::error(400, "Bad Request")
    }

    /// Create a 401 Unauthorized response.
    pub fn unauthorized() -> Self {
        Self::error(401, "Unauthorized")
    }

    /// Create a 403 Forbidden response.
    pub fn forbidden() -> Self {
        Self::error(403, "Forbidden")
    }

    /// Create a 405 Method Not Allowed response.
    pub fn method_not_allowed() -> Self {
        Self::error(405, "Method Not Allowed")
    }

    /// Create a 409 Conflict response.
    pub fn conflict() -> Self {
        Self::error(409, "Conflict")
    }

    /// Create a 422 Unprocessable Entity response.
    pub fn unprocessable_entity() -> Self {
        Self::error(422, "Unprocessable Entity")
    }

    /// Create a 500 Internal Server Error response.
    pub fn internal_server_error() -> Self {
        Self::error(500, "Internal Server Error")
    }

    /// Create a 502 Bad Gateway response.
    pub fn bad_gateway() -> Self {
        Self::error(502, "Bad Gateway")
    }

    /// Create a 503 Service Unavailable response.
    pub fn service_unavailable() -> Self {
        Self::error(503, "Service Unavailable")
    }

    /// Create a 301 Moved Permanently redirect response.
    pub fn moved_permanently(location: impl Into<String>) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 301,
            status_text: "Moved Permanently".to_string(),
            headers: HashMap::from([("Location".to_string(), location.into())]),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a 302 Found (temporary redirect) response.
    pub fn found(location: impl Into<String>) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 302,
            status_text: "Found".to_string(),
            headers: HashMap::from([("Location".to_string(), location.into())]),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Create a 304 Not Modified response.
    pub fn not_modified() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 304,
            status_text: "Not Modified".to_string(),
            headers: HashMap::new(),
            set_cookies: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Add a header to the response.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the response body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.headers
            .insert("Content-Length".to_string(), body.len().to_string());
        self.body = body;
        self
    }

    /// Set the response body with chunked encoding.
    /// This will automatically set the Transfer-Encoding header to "chunked".
    pub fn with_chunked_body(mut self, body: Vec<u8>) -> Self {
        self.headers
            .insert("Transfer-Encoding".to_string(), "chunked".to_string());
        self.headers.remove("Content-Length"); // Remove conflicting header
        self.body = body;
        self
    }

    /// Add a cookie to be set in the response.
    pub fn with_cookie(mut self, cookie: Cookie) -> Self {
        self.set_cookies.push(cookie);
        self
    }

    /// Add a simple cookie with name and value.
    pub fn with_simple_cookie(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_cookie(Cookie::new(name, value))
    }
}

/// HTTP codec for parsing and encoding HTTP requests and responses.
#[derive(Debug, Clone)]
pub struct HttpCodec {
    /// Maximum size for HTTP headers (default: 8KB)
    max_header_size: usize,
    /// Maximum size for HTTP body (default: 1MB)
    max_body_size: usize,
}

impl Default for HttpCodec {
    fn default() -> Self {
        Self {
            max_header_size: 8 * 1024,  // 8KB
            max_body_size: 1024 * 1024, // 1MB
        }
    }
}

impl HttpCodec {
    /// Create a new HTTP codec with default limits.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new HTTP codec with custom limits.
    pub fn with_limits(max_header_size: usize, max_body_size: usize) -> Self {
        Self {
            max_header_size,
            max_body_size,
        }
    }

    /// Parse a chunked HTTP body starting from the given position.
    /// Returns Ok(Some((body, bytes_consumed))) if complete, Ok(None) if more data needed.
    fn parse_chunked_body(
        &self,
        buf: &[u8],
        start: usize,
    ) -> Result<Option<(Vec<u8>, usize)>, HttpCodecError> {
        let mut pos = start;
        let mut body = Vec::new();

        loop {
            // Find the end of the chunk size line (CRLF)
            let remaining = &buf[pos..];
            if let Some(line_end) = remaining.windows(2).position(|w| w == b"\r\n") {
                // Parse chunk size (hexadecimal)
                let chunk_size_str = String::from_utf8_lossy(&remaining[..line_end]);
                let chunk_size_str = chunk_size_str.trim();

                // Parse chunk size, ignoring chunk extensions (after semicolon)
                let chunk_size_part = chunk_size_str.split(';').next().unwrap_or(chunk_size_str);
                let chunk_size = usize::from_str_radix(chunk_size_part, 16)
                    .map_err(|_| HttpCodecError::InvalidRequest)?;

                pos += line_end + 2; // Skip chunk size line and CRLF

                if chunk_size == 0 {
                    // End of chunks - look for final CRLF (and optional trailers)
                    // For simplicity, we'll just skip to the final CRLF
                    if pos + 2 <= buf.len() && &buf[pos..pos + 2] == b"\r\n" {
                        return Ok(Some((body, pos + 2)));
                    } else {
                        // Look for end of trailers (if any) - find the final empty line
                        let remaining = &buf[pos..];
                        if let Some(trailer_end) =
                            remaining.windows(4).position(|w| w == b"\r\n\r\n")
                        {
                            return Ok(Some((body, pos + trailer_end + 4)));
                        } else if remaining.len() >= 2 && &remaining[..2] == b"\r\n" {
                            // Simple case: just final CRLF, no trailers
                            return Ok(Some((body, pos + 2)));
                        } else {
                            return Ok(None); // Need more data
                        }
                    }
                }

                // Check if we have enough data for this chunk + trailing CRLF
                if pos + chunk_size + 2 > buf.len() {
                    return Ok(None); // Need more data
                }

                // Check body size limit
                if body.len() + chunk_size > self.max_body_size {
                    return Err(HttpCodecError::BodyTooLarge);
                }

                // Append chunk data to body
                body.extend_from_slice(&buf[pos..pos + chunk_size]);
                pos += chunk_size;

                // Verify chunk ends with CRLF
                if pos + 2 > buf.len() || &buf[pos..pos + 2] != b"\r\n" {
                    return Err(HttpCodecError::InvalidRequest);
                }
                pos += 2; // Skip trailing CRLF
            } else {
                return Ok(None); // Need more data for chunk size line
            }
        }
    }

    fn parse_request(&self, buf: &[u8]) -> Result<Option<HttpRequest>, HttpCodecError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(buf) {
            Ok(httparse::Status::Complete(header_len)) => {
                let method = req
                    .method
                    .ok_or(HttpCodecError::InvalidRequest)?
                    .to_string();
                let path = req.path.ok_or(HttpCodecError::InvalidRequest)?.to_string();
                let version = match req.version {
                    Some(v) => format!(
                        "HTTP/{}.{}",
                        v,
                        if v == 0 {
                            9
                        } else {
                            if v == 1 { 1 } else { 0 }
                        }
                    ),
                    None => "HTTP/1.1".to_string(), // Default to HTTP/1.1
                };

                let mut headers_map = HashMap::new();
                let mut content_length = 0;
                let mut is_chunked = false;

                for header in req.headers {
                    let name = header.name.to_lowercase(); // HTTP headers are case-insensitive
                    let value = String::from_utf8_lossy(header.value).to_string();

                    match name.as_str() {
                        "content-length" => {
                            content_length = value
                                .parse()
                                .map_err(|_| HttpCodecError::InvalidContentLength)?;
                            if content_length > self.max_body_size {
                                return Err(HttpCodecError::BodyTooLarge);
                            }
                        }
                        "transfer-encoding" => {
                            if value.to_lowercase().contains("chunked") {
                                is_chunked = true;
                            }
                        }
                        _ => {}
                    }

                    // Store original case for the header name from the request
                    headers_map.insert(header.name.to_string(), value);
                }

                // Handle chunked encoding vs content-length
                let body = if is_chunked {
                    // Parse chunked body
                    match self.parse_chunked_body(buf, header_len)? {
                        Some((body, _total_consumed)) => body,
                        None => return Ok(None), // Need more data
                    }
                } else {
                    // Handle fixed content-length body
                    let total_len = header_len + content_length;
                    if buf.len() < total_len {
                        return Ok(None); // Need more data
                    }
                    buf[header_len..total_len].to_vec()
                };

                // Parse query parameters from the path
                let (clean_path, query_params) = HttpRequest::parse_query_params(&path);

                // Parse cookies from the Cookie header
                let cookies = headers_map
                    .get("cookie")
                    .or_else(|| headers_map.get("Cookie")) // Try both cases
                    .map(|cookie_header| Cookie::parse_cookie_header(cookie_header))
                    .unwrap_or_else(HashMap::new);

                Ok(Some(HttpRequest {
                    method,
                    path: clean_path,
                    version,
                    headers: headers_map,
                    query_params,
                    cookies,
                    body,
                }))
            }
            Ok(httparse::Status::Partial) => Ok(None), // Need more data
            Err(_) => Err(HttpCodecError::InvalidRequest),
        }
    }

    fn parse_response(&self, buf: &[u8]) -> Result<Option<HttpResponse>, HttpCodecError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut resp = httparse::Response::new(&mut headers);

        match resp.parse(buf) {
            Ok(httparse::Status::Complete(header_len)) => {
                let status_code = resp.code.ok_or(HttpCodecError::InvalidResponse)?;
                let status_text = resp.reason.unwrap_or("").to_string();
                let version = match resp.version {
                    Some(v) => format!(
                        "HTTP/{}.{}",
                        v,
                        if v == 0 {
                            9
                        } else {
                            if v == 1 { 1 } else { 0 }
                        }
                    ),
                    None => "HTTP/1.1".to_string(), // Default to HTTP/1.1
                };

                let mut headers_map = HashMap::new();
                let mut content_length = 0;
                let mut is_chunked = false;

                for header in resp.headers {
                    let name = header.name.to_lowercase(); // HTTP headers are case-insensitive
                    let value = String::from_utf8_lossy(header.value).to_string();

                    match name.as_str() {
                        "content-length" => {
                            content_length = value
                                .parse()
                                .map_err(|_| HttpCodecError::InvalidContentLength)?;
                            if content_length > self.max_body_size {
                                return Err(HttpCodecError::BodyTooLarge);
                            }
                        }
                        "transfer-encoding" => {
                            if value.to_lowercase().contains("chunked") {
                                is_chunked = true;
                            }
                        }
                        _ => {}
                    }

                    // Store original case for the header name from the response
                    headers_map.insert(header.name.to_string(), value);
                }

                // Handle chunked encoding vs content-length
                let body = if is_chunked {
                    // Parse chunked body
                    match self.parse_chunked_body(buf, header_len)? {
                        Some((body, _total_consumed)) => body,
                        None => return Ok(None), // Need more data
                    }
                } else {
                    // Handle fixed content-length body
                    let total_len = header_len + content_length;
                    if buf.len() < total_len {
                        return Ok(None); // Need more data
                    }
                    buf[header_len..total_len].to_vec()
                };

                Ok(Some(HttpResponse {
                    version,
                    status_code,
                    status_text,
                    headers: headers_map,
                    set_cookies: Vec::new(), /* Set-Cookie headers would be parsed separately if needed */
                    body,
                }))
            }
            Ok(httparse::Status::Partial) => Ok(None), // Need more data
            Err(_) => Err(HttpCodecError::InvalidResponse),
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
            HttpCodecError::InvalidRequest => write!(f, "Invalid HTTP request"),
            HttpCodecError::InvalidResponse => write!(f, "Invalid HTTP response"),
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

/// HTTP codec that can decode requests and encode responses (for servers).
#[derive(Debug, Clone, Default)]
pub struct HttpServerCodec {
    inner: HttpCodec,
}

impl HttpServerCodec {
    /// Create a new HTTP server codec.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new HTTP server codec with custom limits.
    pub fn with_limits(max_header_size: usize, max_body_size: usize) -> Self {
        Self {
            inner: HttpCodec::with_limits(max_header_size, max_body_size),
        }
    }
}

impl Decoder for HttpServerCodec {
    type Item = HttpRequest;
    type Error = HttpCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() > self.inner.max_header_size {
            return Err(HttpCodecError::HeadersTooLarge);
        }

        // Parse the request, which now handles both chunked and content-length internally
        match self.inner.parse_request(src)? {
            Some(request) => {
                // We need to figure out how many bytes were consumed
                // Re-parse to get the consumption information
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let mut req = httparse::Request::new(&mut headers);

                if let Ok(httparse::Status::Complete(header_len)) = req.parse(src) {
                    let mut content_length = 0;
                    let mut is_chunked = false;

                    // Check headers to determine body type
                    for header in req.headers {
                        let name = header.name.to_lowercase();
                        let value = String::from_utf8_lossy(header.value);

                        match name.as_str() {
                            "content-length" => {
                                content_length = value.parse().unwrap_or(0);
                            }
                            "transfer-encoding" => {
                                if value.to_lowercase().contains("chunked") {
                                    is_chunked = true;
                                }
                            }
                            _ => {}
                        }
                    }

                    let bytes_consumed = if is_chunked {
                        // For chunked, parse again to get exact consumption
                        if let Ok(Some((_body, consumed))) =
                            self.inner.parse_chunked_body(src, header_len)
                        {
                            consumed
                        } else {
                            return Err(HttpCodecError::InvalidRequest);
                        }
                    } else {
                        header_len + content_length
                    };

                    src.advance(bytes_consumed);
                    Ok(Some(request))
                } else {
                    Err(HttpCodecError::InvalidRequest)
                }
            }
            None => Ok(None),
        }
    }
}

impl Encoder<HttpResponse> for HttpServerCodec {
    type Error = HttpCodecError;

    fn encode(&mut self, item: HttpResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Status line
        let status_line = format!(
            "{} {} {}\r\n",
            item.version, item.status_code, item.status_text
        );
        dst.put(status_line.as_bytes());

        // Check if this should be chunked encoding
        let is_chunked = item
            .headers
            .get("Transfer-Encoding")
            .map(|v| v.to_lowercase().contains("chunked"))
            .unwrap_or(false);

        // Headers
        for (name, value) in &item.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            dst.put(header_line.as_bytes());
        }

        // Set-Cookie headers
        for cookie in &item.set_cookies {
            let set_cookie_line = format!("Set-Cookie: {}\r\n", cookie.to_set_cookie_header());
            dst.put(set_cookie_line.as_bytes());
        }

        // End of headers
        dst.put(&b"\r\n"[..]);

        // Body
        if is_chunked {
            // Encode body as a single chunk (simple implementation)
            if !item.body.is_empty() {
                let chunk_size = format!("{:x}\r\n", item.body.len());
                dst.put(chunk_size.as_bytes());
                dst.put(&item.body[..]);
                dst.put(&b"\r\n"[..]);
            }
            // End chunk
            dst.put(&b"0\r\n\r\n"[..]);
        } else {
            dst.put(&item.body[..]);
        }

        Ok(())
    }
}

/// HTTP codec that can encode requests and decode responses (for clients).
#[derive(Debug, Clone, Default)]
pub struct HttpClientCodec {
    inner: HttpCodec,
}

impl HttpClientCodec {
    /// Create a new HTTP client codec.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new HTTP client codec with custom limits.
    pub fn with_limits(max_header_size: usize, max_body_size: usize) -> Self {
        Self {
            inner: HttpCodec::with_limits(max_header_size, max_body_size),
        }
    }
}

impl Decoder for HttpClientCodec {
    type Item = HttpResponse;
    type Error = HttpCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() > self.inner.max_header_size {
            return Err(HttpCodecError::HeadersTooLarge);
        }

        // Parse the response, which now handles both chunked and content-length internally
        match self.inner.parse_response(src)? {
            Some(response) => {
                // We need to figure out how many bytes were consumed
                // Re-parse to get the consumption information
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let mut resp = httparse::Response::new(&mut headers);

                if let Ok(httparse::Status::Complete(header_len)) = resp.parse(src) {
                    let mut content_length = 0;
                    let mut is_chunked = false;

                    // Check headers to determine body type
                    for header in resp.headers {
                        let name = header.name.to_lowercase();
                        let value = String::from_utf8_lossy(header.value);

                        match name.as_str() {
                            "content-length" => {
                                content_length = value.parse().unwrap_or(0);
                            }
                            "transfer-encoding" => {
                                if value.to_lowercase().contains("chunked") {
                                    is_chunked = true;
                                }
                            }
                            _ => {}
                        }
                    }

                    let bytes_consumed = if is_chunked {
                        // For chunked, parse again to get exact consumption
                        if let Ok(Some((_body, consumed))) =
                            self.inner.parse_chunked_body(src, header_len)
                        {
                            consumed
                        } else {
                            return Err(HttpCodecError::InvalidResponse);
                        }
                    } else {
                        header_len + content_length
                    };

                    src.advance(bytes_consumed);
                    Ok(Some(response))
                } else {
                    Err(HttpCodecError::InvalidResponse)
                }
            }
            None => Ok(None),
        }
    }
}

impl Encoder<HttpRequest> for HttpClientCodec {
    type Error = HttpCodecError;

    fn encode(&mut self, item: HttpRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Request line
        let request_line = format!("{} {} {}\r\n", item.method, item.path, item.version);
        dst.put(request_line.as_bytes());

        // Headers
        for (name, value) in &item.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            dst.put(header_line.as_bytes());
        }

        // End of headers
        dst.put(&b"\r\n"[..]);

        // Body
        dst.put(&item.body[..]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::Encoder;

    use super::*;

    #[test]
    fn test_http_request_creation() {
        let req = HttpRequest::get("/test");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/test");
        assert_eq!(req.version, "HTTP/1.1");
        assert!(req.body.is_empty());
    }

    #[test]
    fn test_http_methods() {
        // Test all HTTP methods
        let get_req = HttpRequest::get("/test");
        assert_eq!(get_req.method, "GET");
        assert_eq!(get_req.path, "/test");
        assert!(get_req.query_params.is_empty());

        let post_req = HttpRequest::post("/test", b"data".to_vec());
        assert_eq!(post_req.method, "POST");
        assert_eq!(post_req.body, b"data");

        let put_req = HttpRequest::put("/test", b"data".to_vec());
        assert_eq!(put_req.method, "PUT");

        let patch_req = HttpRequest::patch("/test", b"patch_data".to_vec());
        assert_eq!(patch_req.method, "PATCH");
        assert_eq!(patch_req.body, b"patch_data");

        let delete_req = HttpRequest::delete("/test");
        assert_eq!(delete_req.method, "DELETE");

        let head_req = HttpRequest::head("/test");
        assert_eq!(head_req.method, "HEAD");

        let options_req = HttpRequest::options("/test");
        assert_eq!(options_req.method, "OPTIONS");
    }

    #[test]
    fn test_query_parameter_parsing() {
        // Test simple query parameters
        let req = HttpRequest::get("/api/users?id=123&name=john");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.query_params.get("id"), Some(&"123".to_string()));
        assert_eq!(req.query_params.get("name"), Some(&"john".to_string()));

        // Test empty query string
        let req = HttpRequest::get("/api/users");
        assert_eq!(req.path, "/api/users");
        assert!(req.query_params.is_empty());

        // Test URL encoded parameters
        let req = HttpRequest::get("/search?q=hello%20world&category=test%26demo");
        assert_eq!(req.path, "/search");
        assert_eq!(req.query_params.get("q"), Some(&"hello world".to_string()));
        assert_eq!(
            req.query_params.get("category"),
            Some(&"test&demo".to_string())
        );

        // Test parameters without values
        let req = HttpRequest::get("/api/test?debug&verbose&format=json");
        assert_eq!(req.path, "/api/test");
        assert_eq!(req.query_params.get("debug"), Some(&"".to_string()));
        assert_eq!(req.query_params.get("verbose"), Some(&"".to_string()));
        assert_eq!(req.query_params.get("format"), Some(&"json".to_string()));

        // Test plus encoding for spaces
        let req = HttpRequest::get("/search?q=hello+world");
        assert_eq!(req.query_params.get("q"), Some(&"hello world".to_string()));

        // Test empty parameter values
        let req = HttpRequest::get("/api?key1=&key2=value");
        assert_eq!(req.query_params.get("key1"), Some(&"".to_string()));
        assert_eq!(req.query_params.get("key2"), Some(&"value".to_string()));
    }

    #[test]
    fn test_query_parameter_helpers() {
        let mut req = HttpRequest::get("/api/users")
            .with_query_param("id", "123")
            .with_query_param("format", "json");

        // Test getter methods
        assert_eq!(req.get_query_param("id"), Some(&"123".to_string()));
        assert_eq!(req.get_query_param("format"), Some(&"json".to_string()));
        assert_eq!(req.get_query_param("missing"), None);

        // Test has_query_param
        assert!(req.has_query_param("id"));
        assert!(req.has_query_param("format"));
        assert!(!req.has_query_param("missing"));

        // Test full_url reconstruction
        let full_url = req.full_url();
        assert!(full_url.starts_with("/api/users?"));
        assert!(full_url.contains("id=123"));
        assert!(full_url.contains("format=json"));

        // Test with flag-style parameter
        req = req.with_query_param("debug", "");
        assert!(req.has_query_param("debug"));
        assert_eq!(req.get_query_param("debug"), Some(&"".to_string()));
    }

    #[test]
    fn test_url_encoding_decoding() {
        // Test URL encoding
        assert_eq!(HttpRequest::url_encode("hello world"), "hello+world");
        assert_eq!(HttpRequest::url_encode("test&demo"), "test%26demo");
        assert_eq!(
            HttpRequest::url_encode("user@example.com"),
            "user%40example.com"
        );

        // Test URL decoding through request creation
        let req = HttpRequest::get("/search?email=user%40example.com&msg=hello%20world");
        assert_eq!(
            req.query_params.get("email"),
            Some(&"user@example.com".to_string())
        );
        assert_eq!(
            req.query_params.get("msg"),
            Some(&"hello world".to_string())
        );
    }

    #[test]
    fn test_chunked_encoding() -> Result<(), Box<dyn std::error::Error>> {
        use tokio_util::codec::Decoder;

        let mut codec = HttpClientCodec::new();

        // Test chunked response
        let chunked_response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n7\r\nMozilla\r\n9\r\nDeveloper\r\n7\r\nNetwork\r\n0\r\n\r\n";
        let mut buf = BytesMut::from(&chunked_response[..]);

        let response = codec
            .decode(&mut buf)?
            .expect("Should decode chunked response");
        assert_eq!(response.status_code, 200);
        assert_eq!(String::from_utf8(response.body)?, "MozillaDeveloperNetwork");

        // Verify Transfer-Encoding header is preserved
        assert_eq!(
            response.headers.get("Transfer-Encoding"),
            Some(&"chunked".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_chunked_with_extensions() -> Result<(), Box<dyn std::error::Error>> {
        use tokio_util::codec::Decoder;

        let mut codec = HttpClientCodec::new();

        // Test chunked response with chunk extensions (should be ignored)
        let chunked_response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n7;some=extension\r\nMozilla\r\n0\r\n\r\n";
        let mut buf = BytesMut::from(&chunked_response[..]);

        let response = codec
            .decode(&mut buf)?
            .expect("Should decode chunked response with extensions");
        assert_eq!(response.status_code, 200);
        assert_eq!(String::from_utf8(response.body)?, "Mozilla");

        Ok(())
    }

    #[test]
    fn test_chunked_response_encoding() -> Result<(), Box<dyn std::error::Error>> {
        use tokio_util::codec::Encoder;

        let mut codec = HttpServerCodec::new();
        let mut dst = BytesMut::new();

        // Create a chunked response
        let response = HttpResponse::ok().with_chunked_body(b"Hello, World!".to_vec());

        codec.encode(response, &mut dst)?;

        let encoded = String::from_utf8(dst.to_vec())?;

        // Verify the encoded response has chunked format
        assert!(encoded.contains("Transfer-Encoding: chunked"));
        assert!(encoded.contains("d\r\nHello, World!\r\n")); // "d" is hex for 13
        assert!(encoded.ends_with("0\r\n\r\n")); // End chunk

        Ok(())
    }

    #[test]
    fn test_http_response_creation() {
        let resp = HttpResponse::ok();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.status_text, "OK");
        assert_eq!(resp.version, "HTTP/1.1");
    }

    #[test]
    fn test_http_json_response() -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::json!({"message": "hello"});
        let resp = HttpResponse::json(&data)?;
        assert_eq!(resp.status_code, 200);
        assert_eq!(
            resp.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_codec_encode_decode() -> Result<(), Box<dyn std::error::Error>> {
        let mut server_codec = HttpServerCodec::new();
        let mut client_codec = HttpClientCodec::new();

        // Test encoding a request
        let request = HttpRequest::get("/test").with_header("Host", "example.com");
        let mut encoded = BytesMut::new();
        client_codec.encode(request, &mut encoded)?;

        // Test encoding a response
        let response = HttpResponse::ok().with_body(b"Hello, World!".to_vec());
        let mut encoded_resp = BytesMut::new();
        server_codec.encode(response, &mut encoded_resp)?;

        // Basic sanity checks
        let encoded_str = String::from_utf8_lossy(&encoded);
        assert!(encoded_str.contains("GET /test HTTP/1.1"));
        assert!(encoded_str.contains("Host: example.com"));

        let encoded_resp_str = String::from_utf8_lossy(&encoded_resp);
        assert!(encoded_resp_str.contains("HTTP/1.1 200 OK"));
        assert!(encoded_resp_str.contains("Hello, World!"));

        Ok(())
    }

    #[test]
    fn test_dfir_http_processing_pattern() -> Result<(), Box<dyn std::error::Error>> {
        use std::net::SocketAddr;

        use crate::dfir_syntax;
        use crate::util::collect_ready;

        // Create test requests to inject (simulating what would come from bind_http_server)
        let test_requests = vec![
            Ok((
                HttpRequest::get("/"),
                "127.0.0.1:12345".parse::<SocketAddr>().unwrap(),
            )),
            Ok((
                HttpRequest::get("/api/test"),
                "127.0.0.1:12346".parse::<SocketAddr>().unwrap(),
            )),
            Ok((
                HttpRequest::post_json("/api/data", &serde_json::json!({"key": "value"}))?,
                "127.0.0.1:12347".parse::<SocketAddr>().unwrap(),
            )),
        ];

        // Create a test receiver to collect the processed responses
        let (test_response_send, test_response_recv) =
            tokio::sync::mpsc::unbounded_channel::<(HttpResponse, SocketAddr)>();

        let mut dfir_flow = dfir_syntax! {
            // Process HTTP requests through DFIR pipeline
            source_iter(test_requests)
                -> map(|result: Result<(HttpRequest, SocketAddr), HttpCodecError>| {
                    let (request, client_addr) = result.unwrap();
                    println!("Processing {} {} from {}", request.method, request.path, client_addr);

                    // Simple routing logic - this is what a real HTTP server would do
                    let response = match request.path.as_str() {
                        "/" => HttpResponse::ok().with_body(b"Home page".to_vec()),
                        "/api/test" => HttpResponse::json(&serde_json::json!({
                            "message": "Test endpoint",
                            "status": "success"
                        })).unwrap(),
                        "/api/data" => {
                            if request.method == "POST" {
                                HttpResponse::json(&serde_json::json!({
                                    "received": "data",
                                    "echo": String::from_utf8_lossy(&request.body)
                                })).unwrap()
                            } else {
                                HttpResponse::error(405, "Method Not Allowed")
                            }
                        },
                        _ => HttpResponse::error(404, "Not Found")
                            .with_body(b"Page not found".to_vec()),
                    };

                    (response, client_addr)
                })
                // Instead of dest_sink, just send to a regular channel for testing
                -> for_each(|(response, addr)| test_response_send.send((response, addr)).unwrap());
        };

        // Run the DFIR flow
        dfir_flow.run_available();

        // Collect the responses that were sent through the DFIR pipeline
        let responses: Vec<(HttpResponse, SocketAddr)> = collect_ready(
            tokio_stream::wrappers::UnboundedReceiverStream::new(test_response_recv),
        );

        // Verify we got the expected responses
        assert_eq!(responses.len(), 3);

        // Check first response (GET /)
        assert_eq!(responses[0].0.status_code, 200);
        assert_eq!(responses[0].0.body, b"Home page");
        assert_eq!(responses[0].1.port(), 12345);

        // Check second response (GET /api/test)
        assert_eq!(responses[1].0.status_code, 200);
        assert_eq!(
            responses[1].0.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        let json_body: serde_json::Value = serde_json::from_slice(&responses[1].0.body)?;
        assert_eq!(json_body["message"], "Test endpoint");
        assert_eq!(responses[1].1.port(), 12346);

        // Check third response (POST /api/data)
        assert_eq!(responses[2].0.status_code, 200);
        let json_body: serde_json::Value = serde_json::from_slice(&responses[2].0.body)?;
        assert_eq!(json_body["received"], "data");
        assert_eq!(responses[2].1.port(), 12347);

        println!("✅ HTTP request processing works correctly through DFIR pipeline!");
        println!("✅ This demonstrates the DFIR pattern for HTTP processing:");
        println!("   source_stream(request_recv) -> map(route_logic) -> for_each(send_response)");
        println!(
            "✅ For dest_sink examples, see examples/http_server.rs and examples/http_client.rs"
        );

        Ok(())
    }

    #[test]
    fn test_query_parameters_with_different_methods() {
        // Test POST with query parameters
        let req = HttpRequest::post("/api/users?format=json", "test data".as_bytes().to_vec());
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.query_params.get("format"), Some(&"json".to_string()));
        assert_eq!(req.body, "test data".as_bytes().to_vec());

        // Test PUT with query parameters
        let req = HttpRequest::put(
            "/api/users/123?timestamp=123456",
            "update data".as_bytes().to_vec(),
        );
        assert_eq!(req.method, "PUT");
        assert_eq!(req.path, "/api/users/123");
        assert_eq!(
            req.query_params.get("timestamp"),
            Some(&"123456".to_string())
        );
        assert_eq!(req.body, "update data".as_bytes().to_vec());

        // Test PATCH with query parameters
        let req = HttpRequest::patch(
            "/api/users/123?partial=true",
            "patch data".as_bytes().to_vec(),
        );
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.path, "/api/users/123");
        assert_eq!(req.query_params.get("partial"), Some(&"true".to_string()));
        assert_eq!(req.body, "patch data".as_bytes().to_vec());

        // Test DELETE with query parameters
        let req = HttpRequest::delete("/api/users/123?cascade=true");
        assert_eq!(req.method, "DELETE");
        assert_eq!(req.path, "/api/users/123");
        assert_eq!(req.query_params.get("cascade"), Some(&"true".to_string()));

        // Test HEAD with query parameters
        let req = HttpRequest::head("/api/users?count=true");
        assert_eq!(req.method, "HEAD");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.query_params.get("count"), Some(&"true".to_string()));

        // Test OPTIONS with query parameters
        let req = HttpRequest::options("/api?cors=true");
        assert_eq!(req.method, "OPTIONS");
        assert_eq!(req.path, "/api");
        assert_eq!(req.query_params.get("cors"), Some(&"true".to_string()));
    }

    #[test]
    fn test_edge_case_query_parameters() {
        // Test path with only question mark
        let req = HttpRequest::get("/api?");
        assert_eq!(req.path, "/api");
        assert!(req.query_params.is_empty());

        // Test multiple question marks (only first should be treated as separator)
        let req = HttpRequest::get("/api?param=value?other");
        assert_eq!(req.path, "/api");
        assert_eq!(
            req.query_params.get("param"),
            Some(&"value?other".to_string())
        );

        // Test special characters in values
        let req = HttpRequest::get("/api?data=%7B%22key%22%3A%22value%22%7D");
        assert_eq!(
            req.query_params.get("data"),
            Some(&"{\"key\":\"value\"}".to_string())
        );

        // Test duplicate parameter names (should use last value)
        let req = HttpRequest::get("/api?name=first&name=second");
        assert_eq!(req.query_params.get("name"), Some(&"second".to_string()));

        // Test parameter names with special characters
        let req = HttpRequest::get("/api?my-param=value&my_param2=value2");
        assert_eq!(req.query_params.get("my-param"), Some(&"value".to_string()));
        assert_eq!(
            req.query_params.get("my_param2"),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_url_encode_decode_edge_cases() {
        // Test encoding special characters
        assert_eq!(
            HttpRequest::url_encode("!#$&'()*+,/:;=?@[]"),
            "%21%23%24%26%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D"
        );

        // Test already encoded strings
        assert_eq!(
            HttpRequest::url_encode("already%20encoded"),
            "already%2520encoded"
        );

        // Test empty string
        assert_eq!(HttpRequest::url_encode(""), "");

        // Test Unicode characters
        assert_eq!(HttpRequest::url_encode("café"), "caf%C3%A9");
    }

    #[test]
    fn test_request_building_flow() {
        // Test building request with multiple operations
        let req = HttpRequest::get("/api/search")
            .with_query_param("q", "rust programming")
            .with_query_param("limit", "10")
            .with_header("User-Agent", "test-client")
            .with_header("Accept", "application/json");

        assert_eq!(req.path, "/api/search");
        assert_eq!(
            req.query_params.get("q"),
            Some(&"rust programming".to_string())
        );
        assert_eq!(req.query_params.get("limit"), Some(&"10".to_string()));
        assert_eq!(
            req.headers.get("User-Agent"),
            Some(&"test-client".to_string())
        );
        assert_eq!(
            req.headers.get("Accept"),
            Some(&"application/json".to_string())
        );

        // Test that full_url includes all parameters
        let full_url = req.full_url();
        assert!(full_url.starts_with("/api/search?"));
        assert!(full_url.contains("q=rust+programming"));
        assert!(full_url.contains("limit=10"));
    }

    #[test]
    fn test_status_code_helpers() {
        // Test success responses
        let ok_resp = HttpResponse::ok();
        assert_eq!(ok_resp.status_code, 200);
        assert_eq!(ok_resp.status_text, "OK");

        let created_resp = HttpResponse::created();
        assert_eq!(created_resp.status_code, 201);
        assert_eq!(created_resp.status_text, "Created");

        let no_content_resp = HttpResponse::no_content();
        assert_eq!(no_content_resp.status_code, 204);
        assert_eq!(no_content_resp.status_text, "No Content");

        // Test redirect responses
        let moved_resp = HttpResponse::moved_permanently("/new-location");
        assert_eq!(moved_resp.status_code, 301);
        assert_eq!(moved_resp.status_text, "Moved Permanently");
        assert_eq!(
            moved_resp.headers.get("Location"),
            Some(&"/new-location".to_string())
        );

        let found_resp = HttpResponse::found("https://example.com/redirect");
        assert_eq!(found_resp.status_code, 302);
        assert_eq!(found_resp.status_text, "Found");
        assert_eq!(
            found_resp.headers.get("Location"),
            Some(&"https://example.com/redirect".to_string())
        );

        let not_modified_resp = HttpResponse::not_modified();
        assert_eq!(not_modified_resp.status_code, 304);
        assert_eq!(not_modified_resp.status_text, "Not Modified");

        // Test client error responses
        let bad_request_resp = HttpResponse::bad_request();
        assert_eq!(bad_request_resp.status_code, 400);
        assert_eq!(bad_request_resp.status_text, "Bad Request");

        let unauthorized_resp = HttpResponse::unauthorized();
        assert_eq!(unauthorized_resp.status_code, 401);
        assert_eq!(unauthorized_resp.status_text, "Unauthorized");

        let forbidden_resp = HttpResponse::forbidden();
        assert_eq!(forbidden_resp.status_code, 403);
        assert_eq!(forbidden_resp.status_text, "Forbidden");

        let not_found_resp = HttpResponse::not_found();
        assert_eq!(not_found_resp.status_code, 404);
        assert_eq!(not_found_resp.status_text, "Not Found");

        let method_not_allowed_resp = HttpResponse::method_not_allowed();
        assert_eq!(method_not_allowed_resp.status_code, 405);
        assert_eq!(method_not_allowed_resp.status_text, "Method Not Allowed");

        let conflict_resp = HttpResponse::conflict();
        assert_eq!(conflict_resp.status_code, 409);
        assert_eq!(conflict_resp.status_text, "Conflict");

        let unprocessable_resp = HttpResponse::unprocessable_entity();
        assert_eq!(unprocessable_resp.status_code, 422);
        assert_eq!(unprocessable_resp.status_text, "Unprocessable Entity");

        // Test server error responses
        let server_error_resp = HttpResponse::internal_server_error();
        assert_eq!(server_error_resp.status_code, 500);
        assert_eq!(server_error_resp.status_text, "Internal Server Error");

        let bad_gateway_resp = HttpResponse::bad_gateway();
        assert_eq!(bad_gateway_resp.status_code, 502);
        assert_eq!(bad_gateway_resp.status_text, "Bad Gateway");

        let service_unavailable_resp = HttpResponse::service_unavailable();
        assert_eq!(service_unavailable_resp.status_code, 503);
        assert_eq!(service_unavailable_resp.status_text, "Service Unavailable");
    }

    #[test]
    fn test_patch_json_functionality() -> Result<(), Box<dyn std::error::Error>> {
        // Test PATCH with JSON body
        let patch_data = serde_json::json!({
            "email": "newemail@example.com",
            "preferences": {
                "theme": "dark",
                "notifications": true
            }
        });

        let req = HttpRequest::patch_json("/api/users/123", &patch_data)?;
        assert_eq!(req.method, "PATCH");
        assert_eq!(req.path, "/api/users/123");
        assert_eq!(
            req.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            req.headers.get("Content-Length"),
            Some(&req.body.len().to_string())
        );

        // Verify the JSON body
        let parsed_body: serde_json::Value = serde_json::from_slice(&req.body)?;
        assert_eq!(parsed_body["email"], "newemail@example.com");
        assert_eq!(parsed_body["preferences"]["theme"], "dark");
        assert_eq!(parsed_body["preferences"]["notifications"], true);

        // Test PATCH JSON with query parameters
        let req = HttpRequest::patch_json("/api/users/123?validate=true", &patch_data)?;
        assert_eq!(req.path, "/api/users/123");
        assert_eq!(req.query_params.get("validate"), Some(&"true".to_string()));

        Ok(())
    }

    #[test]
    fn test_status_code_helpers_with_chaining() {
        // Test that status code helpers work with method chaining
        let api_error = HttpResponse::bad_request()
            .with_header("Content-Type", "application/json")
            .with_body(br#"{"error": "Invalid input"}"#.to_vec());

        assert_eq!(api_error.status_code, 400);
        assert_eq!(
            api_error.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(api_error.body, br#"{"error": "Invalid input"}"#);

        // Test redirect with additional headers
        let redirect = HttpResponse::found("/login")
            .with_header("Cache-Control", "no-cache")
            .with_header("Set-Cookie", "session=expired; Max-Age=0");

        assert_eq!(redirect.status_code, 302);
        assert_eq!(
            redirect.headers.get("Location"),
            Some(&"/login".to_string())
        );
        assert_eq!(
            redirect.headers.get("Cache-Control"),
            Some(&"no-cache".to_string())
        );
        assert!(redirect.headers.contains_key("Set-Cookie"));

        // Test created response with location header
        let created_resource = HttpResponse::created()
            .with_header("Location", "/api/users/123")
            .with_header("Content-Type", "application/json")
            .with_body(br#"{"id": 123, "status": "created"}"#.to_vec());

        assert_eq!(created_resource.status_code, 201);
        assert_eq!(
            created_resource.headers.get("Location"),
            Some(&"/api/users/123".to_string())
        );
    }

    #[test]
    fn test_cookie_creation_and_attributes() {
        // Test basic cookie creation
        let cookie = Cookie::new("session_id", "abc123");
        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
        assert_eq!(cookie.domain, None);

        // Test cookie with all attributes
        let cookie = Cookie::new("auth_token", "xyz789")
            .with_domain("example.com")
            .with_path("/app")
            .with_max_age(3600)
            .with_expires("Wed, 21 Oct 2025 07:28:00 GMT")
            .secure()
            .http_only()
            .with_same_site(SameSite::Strict);

        assert_eq!(cookie.domain, Some("example.com".to_string()));
        assert_eq!(cookie.path, Some("/app".to_string()));
        assert_eq!(cookie.max_age, Some(3600));
        assert_eq!(
            cookie.expires,
            Some("Wed, 21 Oct 2025 07:28:00 GMT".to_string())
        );
        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, Some(SameSite::Strict));
    }

    #[test]
    fn test_cookie_header_parsing() {
        // Test parsing single cookie
        let header = "session_id=abc123";
        let cookies = Cookie::parse_cookie_header(header);
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));

        // Test parsing multiple cookies
        let header = "session_id=abc123; user_pref=dark_mode; cart_items=3";
        let cookies = Cookie::parse_cookie_header(header);
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user_pref"), Some(&"dark_mode".to_string()));
        assert_eq!(cookies.get("cart_items"), Some(&"3".to_string()));

        // Test parsing with spaces and edge cases
        let header = " session_id = abc123 ; user_pref=dark_mode; empty=; name_only";
        let cookies = Cookie::parse_cookie_header(header);
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_string()));
        assert_eq!(cookies.get("user_pref"), Some(&"dark_mode".to_string()));
        assert_eq!(cookies.get("empty"), Some(&"".to_string()));
        // name_only without = should be ignored
        assert!(!cookies.contains_key("name_only"));

        // Test empty header
        let cookies = Cookie::parse_cookie_header("");
        assert!(cookies.is_empty());
    }

    #[test]
    fn test_cookie_set_cookie_header_generation() {
        // Test basic cookie
        let cookie = Cookie::new("session_id", "abc123");
        assert_eq!(cookie.to_set_cookie_header(), "session_id=abc123");

        // Test cookie with domain and path
        let cookie = Cookie::new("auth_token", "xyz789")
            .with_domain("example.com")
            .with_path("/app");
        assert_eq!(
            cookie.to_set_cookie_header(),
            "auth_token=xyz789; Domain=example.com; Path=/app"
        );

        // Test cookie with all attributes
        let cookie = Cookie::new("secure_token", "secure123")
            .with_domain(".example.com")
            .with_path("/")
            .with_max_age(86400)
            .with_expires("Thu, 01 Jan 1970 00:00:01 GMT")
            .secure()
            .http_only()
            .with_same_site(SameSite::Lax);

        let header = cookie.to_set_cookie_header();
        assert!(header.contains("secure_token=secure123"));
        assert!(header.contains("Domain=.example.com"));
        assert!(header.contains("Path=/"));
        assert!(header.contains("Max-Age=86400"));
        assert!(header.contains("Expires=Thu, 01 Jan 1970 00:00:01 GMT"));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("SameSite=Lax"));

        // Test SameSite variants
        let strict_cookie = Cookie::new("test", "value").with_same_site(SameSite::Strict);
        assert!(
            strict_cookie
                .to_set_cookie_header()
                .contains("SameSite=Strict")
        );

        let none_cookie = Cookie::new("test", "value").with_same_site(SameSite::None);
        assert!(none_cookie.to_set_cookie_header().contains("SameSite=None"));
    }

    #[test]
    fn test_http_request_cookie_methods() {
        // Create request with cookies via constructor (simulating parsed request)
        let mut req = HttpRequest::get("/test");
        req.cookies
            .insert("session_id".to_string(), "abc123".to_string());
        req.cookies
            .insert("user_pref".to_string(), "dark_mode".to_string());

        // Test cookie accessor methods
        assert_eq!(req.get_cookie("session_id"), Some(&"abc123".to_string()));
        assert_eq!(req.get_cookie("user_pref"), Some(&"dark_mode".to_string()));
        assert_eq!(req.get_cookie("missing"), None);

        assert!(req.has_cookie("session_id"));
        assert!(req.has_cookie("user_pref"));
        assert!(!req.has_cookie("missing"));
    }

    #[test]
    fn test_http_response_cookie_methods() {
        // Test adding simple cookies
        let resp = HttpResponse::ok()
            .with_simple_cookie("session_id", "abc123")
            .with_simple_cookie("user_pref", "dark_mode");

        assert_eq!(resp.set_cookies.len(), 2);
        assert_eq!(resp.set_cookies[0].name, "session_id");
        assert_eq!(resp.set_cookies[0].value, "abc123");
        assert_eq!(resp.set_cookies[1].name, "user_pref");
        assert_eq!(resp.set_cookies[1].value, "dark_mode");

        // Test adding complex cookie
        let complex_cookie = Cookie::new("auth_token", "xyz789")
            .with_domain("example.com")
            .secure()
            .http_only();

        let resp = HttpResponse::ok().with_cookie(complex_cookie);
        assert_eq!(resp.set_cookies.len(), 1);
        assert_eq!(resp.set_cookies[0].name, "auth_token");
        assert_eq!(resp.set_cookies[0].domain, Some("example.com".to_string()));
        assert!(resp.set_cookies[0].secure);
        assert!(resp.set_cookies[0].http_only);
    }

    #[test]
    fn test_cookie_codec_integration() -> Result<(), Box<dyn std::error::Error>> {
        use tokio_util::codec::{Decoder, Encoder};

        // Test request with cookies
        let mut server_codec = HttpServerCodec::new();

        // Simulate an HTTP request with cookies
        let request_with_cookies = b"GET /test HTTP/1.1\r\nHost: example.com\r\nCookie: session_id=abc123; user_pref=dark_mode\r\n\r\n";
        let mut buf = BytesMut::from(&request_with_cookies[..]);

        let request = server_codec
            .decode(&mut buf)?
            .expect("Should decode request");
        assert_eq!(request.path, "/test");
        assert_eq!(
            request.get_cookie("session_id"),
            Some(&"abc123".to_string())
        );
        assert_eq!(
            request.get_cookie("user_pref"),
            Some(&"dark_mode".to_string())
        );

        // Test response with Set-Cookie headers
        let response = HttpResponse::ok()
            .with_simple_cookie("new_session", "def456")
            .with_cookie(Cookie::new("secure_token", "ghi789").secure().http_only())
            .with_body(b"Success".to_vec());

        let mut encoded_resp = BytesMut::new();
        server_codec.encode(response, &mut encoded_resp)?;

        let encoded_str = String::from_utf8_lossy(&encoded_resp);
        assert!(encoded_str.contains("Set-Cookie: new_session=def456\r\n"));
        assert!(encoded_str.contains("Set-Cookie: secure_token=ghi789; Secure; HttpOnly\r\n"));
        assert!(encoded_str.contains("Success"));

        Ok(())
    }

    #[test]
    fn test_dfir_cookie_processing_pattern() -> Result<(), Box<dyn std::error::Error>> {
        use std::net::SocketAddr;

        use crate::dfir_syntax;
        use crate::util::collect_ready;

        // Create test requests with cookies
        let test_requests = vec![
            Ok((
                {
                    let mut req = HttpRequest::get("/login");
                    req.cookies
                        .insert("session_id".to_string(), "expired123".to_string());
                    req
                },
                "127.0.0.1:12345".parse::<SocketAddr>().unwrap(),
            )),
            Ok((
                {
                    let mut req = HttpRequest::post("/api/data", b"test".to_vec());
                    req.cookies
                        .insert("auth_token".to_string(), "valid456".to_string());
                    req.cookies
                        .insert("user_pref".to_string(), "dark_mode".to_string());
                    req
                },
                "127.0.0.1:12346".parse::<SocketAddr>().unwrap(),
            )),
        ];

        let (test_response_send, test_response_recv) =
            tokio::sync::mpsc::unbounded_channel::<(HttpResponse, SocketAddr)>();

        let mut dfir_flow = dfir_syntax! {
            source_iter(test_requests)
                -> map(|result: Result<(HttpRequest, SocketAddr), HttpCodecError>| {
                    let (request, client_addr) = result.unwrap();

                    // Cookie-based routing and authentication
                    let response = match request.path.as_str() {
                        "/login" => {
                            if request.has_cookie("session_id") {
                                // Session exists, refresh it
                                HttpResponse::ok()
                                    .with_simple_cookie("session_id", "new789")
                                    .with_cookie(Cookie::new("csrf_token", "csrf123").secure().http_only())
                                    .with_body(b"Session refreshed".to_vec())
                            } else {
                                // No session, create new one
                                HttpResponse::ok()
                                    .with_simple_cookie("session_id", "first123")
                                    .with_body(b"New session created".to_vec())
                            }
                        }
                        "/api/data" => {
                            if request.get_cookie("auth_token") == Some(&"valid456".to_string()) {
                                let user_pref = request.get_cookie("user_pref")
                                    .map(|s| s.as_str())
                                    .unwrap_or("light_mode");
                                HttpResponse::json(&serde_json::json!({
                                    "data": "secure data",
                                    "theme": user_pref
                                })).unwrap()
                            } else {
                                HttpResponse::unauthorized()
                                    .with_body(b"Invalid authentication".to_vec())
                            }
                        }
                        _ => HttpResponse::not_found()
                    };

                    (response, client_addr)
                })
                -> for_each(|(response, addr)| test_response_send.send((response, addr)).unwrap());
        };

        dfir_flow.run_available();

        let responses: Vec<(HttpResponse, SocketAddr)> = collect_ready(
            tokio_stream::wrappers::UnboundedReceiverStream::new(test_response_recv),
        );

        assert_eq!(responses.len(), 2);

        // Check login response (session refresh)
        assert_eq!(responses[0].0.status_code, 200);
        assert_eq!(responses[0].0.body, b"Session refreshed");
        assert_eq!(responses[0].0.set_cookies.len(), 2);
        assert_eq!(responses[0].0.set_cookies[0].name, "session_id");
        assert_eq!(responses[0].0.set_cookies[0].value, "new789");
        assert_eq!(responses[0].0.set_cookies[1].name, "csrf_token");
        assert!(responses[0].0.set_cookies[1].secure);
        assert!(responses[0].0.set_cookies[1].http_only);

        // Check API response (authenticated with preferences)
        assert_eq!(responses[1].0.status_code, 200);
        let json_body: serde_json::Value = serde_json::from_slice(&responses[1].0.body)?;
        assert_eq!(json_body["data"], "secure data");
        assert_eq!(json_body["theme"], "dark_mode");

        println!("✅ Cookie support working correctly through DFIR pipeline!");
        println!("✅ Supports: cookie parsing, authentication, session management, preferences");

        Ok(())
    }
}
