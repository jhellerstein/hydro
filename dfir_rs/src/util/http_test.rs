#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

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
        client_codec.encode(request.clone(), &mut encoded)?;

        // Test encoding a response
        let response = HttpResponse::ok().with_body(b"Hello, World!".to_vec());
        let mut encoded_resp = BytesMut::new();
        server_codec.encode(response.clone(), &mut encoded_resp)?;

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
