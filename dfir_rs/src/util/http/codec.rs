//! HTTP codec for parsing and encoding HTTP messages.

use std::collections::HashMap;

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::util::http::encoding::parse_query_string;
use crate::util::http::types::{Cookie, HttpCodecError, HttpRequest, HttpResponse};

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
            // Find chunk size line
            if let Some(crlf_pos) = Self::find_crlf(&buf[pos..]) {
                let chunk_size_line = &buf[pos..pos + crlf_pos];
                let chunk_size_str = std::str::from_utf8(chunk_size_line)
                    .map_err(|_| HttpCodecError::InvalidRequest)?;

                // Parse chunk size (ignore extensions after ';')
                let chunk_size_part = chunk_size_str.split(';').next().unwrap_or(chunk_size_str);
                let chunk_size = usize::from_str_radix(chunk_size_part, 16)
                    .map_err(|_| HttpCodecError::InvalidRequest)?;

                pos += crlf_pos + 2; // Skip past CRLF

                if chunk_size == 0 {
                    // End of chunks - skip final CRLF
                    if buf.len() >= pos + 2 {
                        pos += 2;
                        return Ok(Some((body, pos)));
                    } else {
                        return Ok(None); // Need more data for final CRLF
                    }
                }

                // Read chunk data
                if buf.len() < pos + chunk_size + 2 {
                    return Ok(None); // Need more data
                }

                body.extend_from_slice(&buf[pos..pos + chunk_size]);
                pos += chunk_size + 2; // Skip chunk data and trailing CRLF
            } else {
                return Ok(None); // Need more data
            }
        }
    }

    fn find_crlf(buf: &[u8]) -> Option<usize> {
        buf.windows(2).position(|window| window == b"\r\n")
    }

    /// Parse an HTTP request from a byte buffer.
    pub fn parse_request(&self, buf: &[u8]) -> Result<Option<HttpRequest>, HttpCodecError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(buf) {
            Ok(httparse::Status::Complete(header_len)) => {
                let method = req.method.unwrap().to_string();
                let path = req.path.unwrap().to_string();
                let version = match req.version.unwrap() {
                    0 => "HTTP/1.0".to_string(),
                    1 => "HTTP/1.1".to_string(),
                    v => format!("HTTP/1.{}", v), // Future-proof for other minor versions
                };

                let mut headers_map = HashMap::new();
                let mut cookies = HashMap::new();
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
                        "cookie" => {
                            // Parse cookies from the Cookie header
                            cookies.extend(Cookie::parse_cookie_header(&value));
                        }
                        _ => {}
                    }

                    // Store original case for the header name from the request
                    headers_map.insert(header.name.to_string(), value);
                }

                // Parse query parameters from path
                let (clean_path, query_params) = if let Some(query_start) = path.find('?') {
                    let (path_part, query_part) = path.split_at(query_start);
                    let query_string = &query_part[1..]; // Skip the '?' character
                    let params = parse_query_string(query_string);
                    (path_part.to_string(), params)
                } else {
                    (path, HashMap::new())
                };

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

                // Parse form data if this is a form request
                let mut form_data = HashMap::new();
                if let Some(content_type) = headers_map
                    .get("content-type")
                    .or_else(|| headers_map.get("Content-Type"))
                {
                    if content_type.starts_with("application/x-www-form-urlencoded") {
                        if let Ok(body_str) = std::str::from_utf8(&body) {
                            form_data = crate::util::http::encoding::parse_form_string(body_str);
                        }
                    }
                }

                Ok(Some(HttpRequest {
                    method,
                    path: clean_path,
                    version,
                    headers: headers_map,
                    body,
                    query_params,
                    cookies,
                    form_data,
                }))
            }
            Ok(httparse::Status::Partial) => Ok(None), // Need more data
            Err(_) => Err(HttpCodecError::InvalidRequest),
        }
    }

    /// Parse an HTTP response from a byte buffer.
    pub fn parse_response(&self, buf: &[u8]) -> Result<Option<HttpResponse>, HttpCodecError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut resp = httparse::Response::new(&mut headers);

        match resp.parse(buf) {
            Ok(httparse::Status::Complete(header_len)) => {
                let version = format!("HTTP/{}.{}", resp.version.unwrap(), resp.version.unwrap());
                let status_code = resp.code.unwrap();
                let status_text = resp.reason.unwrap_or("").to_string();

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
    use tokio_util::codec::{Decoder, Encoder};

    use super::*;

    #[test]
    fn test_chunked_encoding() -> Result<(), Box<dyn std::error::Error>> {
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
}
