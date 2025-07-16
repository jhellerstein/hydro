#![cfg(not(target_arch = "wasm32"))]

//! HTTP support for DFIR.
//!
//! This module provides HTTP request and response types along with codecs for parsing
//! HTTP messages over TCP connections. It's designed to work seamlessly with DFIR's
//! stream processing model.
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
//!
//! # async fn example() {
//! let (response_send, request_recv, addr) =
//!     bind_http_server("127.0.0.1:8080".parse().unwrap()).await;
//!
//! // In your DFIR graph:
//! // source_stream(request_recv)
//! //   -> map(|result| result.unwrap())  // Handle errors appropriately in real code
//! //   -> demux(|(request, client_addr), var_args!(home, not_found)| {
//! //       match request.path.as_str() {
//! //           "/" => home.give((request, client_addr)),
//! //           _ => not_found.give((request, client_addr)),
//! //       }
//! //   });
//! //
//! // Handle home page requests
//! // requests[home] -> map(|(_, client_addr)| {
//! //     (HttpResponse::ok().with_body(b"Hello, World!".to_vec()), client_addr)
//! // }) -> dest_sink(response_send);
//! //
//! // Handle 404s
//! // requests[not_found] -> map(|(_, client_addr)| {
//! //     (HttpResponse::not_found(), client_addr)
//! // }) -> dest_sink(response_send);
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
//!
//! # async fn example() {
//! let (request_send, response_recv) = connect_http_client();
//!
//! // In your DFIR graph:
//! // source_iter([HttpRequest::get("/")])
//! //   -> map(|req| (req, "127.0.0.1:8080".parse().unwrap()))
//! //   -> dest_sink(request_send);
//! //
//! // source_stream(response_recv)
//! //   -> demux(|result, var_args!(success, error)| {
//! //       match result {
//! //           Ok((response, addr)) => success.give((response, addr)),
//! //           Err(e) => error.give(e),
//! //       }
//! //   });
//! //
//! // Handle successful responses
//! // responses[success] -> for_each(|(response, addr)| {
//! //     println!("Got response: {} from {}", response.status_code, addr);
//! // });
//! //
//! // Handle errors
//! // responses[error] -> for_each(|e| {
//! //     eprintln!("HTTP error: {}", e);
//! // });
//! # }
//! ```

use std::collections::HashMap;
use std::fmt;

use bytes::{Buf, BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder};

/// A simple HTTP request representation for DFIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
    /// HTTP version (e.g., "HTTP/1.1")
    pub version: String,
    /// Request headers
    pub headers: HashMap<String, String>,
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
    /// Response body
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Create a simple GET request.
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
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

        Ok(Self {
            method: "POST".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers,
            body,
        })
    }

    /// Create a POST request with arbitrary body.
    pub fn post(path: impl Into<String>, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());

        Self {
            method: "POST".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers,
            body,
        }
    }

    /// Create a PUT request.
    pub fn put(path: impl Into<String>, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());

        Self {
            method: "PUT".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers,
            body,
        }
    }

    /// Create a DELETE request.
    pub fn delete(path: impl Into<String>) -> Self {
        Self {
            method: "DELETE".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a HEAD request.
    pub fn head(path: impl Into<String>) -> Self {
        Self {
            method: "HEAD".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create an OPTIONS request.
    pub fn options(path: impl Into<String>) -> Self {
        Self {
            method: "OPTIONS".to_string(),
            path: path.into(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Add a header to the request.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
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
            body: Vec::new(),
        }
    }

    /// Create a 404 Not Found response.
    pub fn not_found() -> Self {
        Self::error(404, "Not Found")
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
                    Some(v) => format!("HTTP/{}.{}", v, if v == 0 { 9 } else { if v == 1 { 1 } else { 0 } }),
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
                        },
                        "transfer-encoding" => {
                            if value.to_lowercase().contains("chunked") {
                                is_chunked = true;
                            }
                        },
                        _ => {}
                    }

                    // Store original case for the header name from the request
                    headers_map.insert(header.name.to_string(), value);
                }

                // Handle chunked encoding vs content-length
                if is_chunked {
                    // For chunked encoding, we'd need to parse chunks
                    // For now, return an error as chunked encoding is not yet supported
                    return Err(HttpCodecError::UnsupportedEncoding);
                }

                let total_len = header_len + content_length;
                if buf.len() < total_len {
                    return Ok(None); // Need more data
                }

                let body = buf[header_len..total_len].to_vec();

                Ok(Some(HttpRequest {
                    method,
                    path,
                    version,
                    headers: headers_map,
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
                let version = format!(
                    "HTTP/{}.{}",
                    resp.version.unwrap_or(1),
                    resp.version.unwrap_or(1)
                );

                let mut headers_map = HashMap::new();
                let mut content_length = 0;

                for header in resp.headers {
                    let name = header.name.to_string();
                    let value = String::from_utf8_lossy(header.value).to_string();

                    if name.to_lowercase() == "content-length" {
                        content_length = value
                            .parse()
                            .map_err(|_| HttpCodecError::InvalidContentLength)?;
                        if content_length > self.max_body_size {
                            return Err(HttpCodecError::BodyTooLarge);
                        }
                    }

                    headers_map.insert(name, value);
                }

                let total_len = header_len + content_length;
                if buf.len() < total_len {
                    return Ok(None); // Need more data
                }

                let body = buf[header_len..total_len].to_vec();

                Ok(Some(HttpResponse {
                    version,
                    status_code,
                    status_text,
                    headers: headers_map,
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
    /// Unsupported encoding (e.g., chunked transfer encoding)
    UnsupportedEncoding,
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
            HttpCodecError::UnsupportedEncoding => write!(f, "Unsupported HTTP encoding"),
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

        match self.inner.parse_request(src)? {
            Some(request) => {
                // Calculate how many bytes to advance
                let content_length = request
                    .headers
                    .get("content-length")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);

                // Find the end of headers (double CRLF)
                if let Some(header_end) = src.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header_len = header_end + 4;
                    let total_len = header_len + content_length;
                    src.advance(total_len);
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

        match self.inner.parse_response(src)? {
            Some(response) => {
                // Calculate how many bytes to advance
                let content_length = response
                    .headers
                    .get("content-length")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);

                // Find the end of headers (double CRLF)
                if let Some(header_end) = src.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header_len = header_end + 4;
                    let total_len = header_len + content_length;
                    src.advance(total_len);
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

        let post_req = HttpRequest::post("/test", b"data".to_vec());
        assert_eq!(post_req.method, "POST");
        assert_eq!(post_req.body, b"data");

        let put_req = HttpRequest::put("/test", b"data".to_vec());
        assert_eq!(put_req.method, "PUT");

        let delete_req = HttpRequest::delete("/test");
        assert_eq!(delete_req.method, "DELETE");

        let head_req = HttpRequest::head("/test");
        assert_eq!(head_req.method, "HEAD");

        let options_req = HttpRequest::options("/test");
        assert_eq!(options_req.method, "OPTIONS");
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
}
