//! HTTP response handling and builder methods.

use std::collections::HashMap;
use serde::Serialize;
use crate::util::http::types::{HttpResponse, Cookie};

impl HttpResponse {
    /// Create a simple 200 OK response.
    pub fn ok() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            set_cookies: Vec::new(),
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
            set_cookies: Vec::new(),
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
            set_cookies: Vec::new(),
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
            body: Vec::new(),
            set_cookies: Vec::new(),
        }
    }

    /// Create a 204 No Content response.
    pub fn no_content() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 204,
            status_text: "No Content".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            set_cookies: Vec::new(),
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
            body: Vec::new(),
            set_cookies: Vec::new(),
        }
    }

    /// Create a 302 Found (temporary redirect) response.
    pub fn found(location: impl Into<String>) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 302,
            status_text: "Found".to_string(),
            headers: HashMap::from([("Location".to_string(), location.into())]),
            body: Vec::new(),
            set_cookies: Vec::new(),
        }
    }

    /// Create a 304 Not Modified response.
    pub fn not_modified() -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status_code: 304,
            status_text: "Not Modified".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            set_cookies: Vec::new(),
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

    /// Get a header value by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_response_cookie_methods() {
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
}
