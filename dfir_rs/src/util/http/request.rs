//! HTTP request handling and builder methods.

use std::collections::HashMap;
use serde::Serialize;
use crate::util::http::types::HttpRequest;
use crate::util::http::encoding::{url_encode, parse_query_string, parse_form_string, encode_form_data};

impl HttpRequest {
    /// Create a simple GET request.
    pub fn get(path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "GET".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body,
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body,
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body,
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body,
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body,
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body: Vec::new(),
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body: Vec::new(),
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
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
            body: Vec::new(),
            query_params,
            cookies: HashMap::new(),
            form_data: HashMap::new(),
        }
    }

    /// Create a POST request with form data.
    pub fn post_form(path: impl Into<String>, form_data: HashMap<String, String>) -> Self {
        let body = encode_form_data(&form_data);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "POST".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            body,
            query_params,
            cookies: HashMap::new(),
            form_data,
        }
    }

    /// Create a PATCH request with form data.
    pub fn patch_form(path: impl Into<String>, form_data: HashMap<String, String>) -> Self {
        let body = encode_form_data(&form_data);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());

        let path_str = path.into();
        let (clean_path, query_params) = Self::parse_query_params(&path_str);

        Self {
            method: "PATCH".to_string(),
            path: clean_path,
            version: "HTTP/1.1".to_string(),
            headers,
            body,
            query_params,
            cookies: HashMap::new(),
            form_data,
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
            let query_string = crate::util::http::encoding::build_query_string(&self.query_params);
            format!("{}?{}", self.path, query_string)
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

    /// Get a form field value by name.
    pub fn get_form_field(&self, name: &str) -> Option<&String> {
        self.form_data.get(name)
    }

    /// Check if a form field exists.
    pub fn has_form_field(&self, name: &str) -> bool {
        self.form_data.contains_key(name)
    }

    /// Parse form data from the request body.
    /// This method parses application/x-www-form-urlencoded content.
    pub fn parse_form_data(&mut self) {
        // Check if this is a form content type
        if let Some(content_type) = self.headers.get("content-type")
            .or_else(|| self.headers.get("Content-Type")) {
            if content_type.starts_with("application/x-www-form-urlencoded") {
                if let Ok(body_str) = std::str::from_utf8(&self.body) {
                    self.form_data = parse_form_string(body_str);
                }
            }
        }
    }

    /// Simple URL encoding for query parameters (re-exported from encoding module).
    pub fn url_encode(s: &str) -> String {
        url_encode(s)
    }

    /// Parse form data string (re-exported from encoding module).
    pub fn parse_form_string(form_str: &str) -> HashMap<String, String> {
        parse_form_string(form_str)
    }

    /// Encode form data (re-exported from encoding module).
    pub fn encode_form_data(form_data: &HashMap<String, String>) -> Vec<u8> {
        encode_form_data(form_data)
    }

    /// Parse query parameters from a URL path.
    fn parse_query_params(path: &str) -> (String, HashMap<String, String>) {
        if let Some(query_start) = path.find('?') {
            let (path_part, query_part) = path.split_at(query_start);
            let query_string = &query_part[1..]; // Skip the '?' character
            let params = parse_query_string(query_string);
            (path_part.to_string(), params)
        } else {
            (path.to_string(), HashMap::new())
        }
    }
}

#[cfg(test)]
mod tests {
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
    }

    #[test]
    fn test_form_methods() -> Result<(), Box<dyn std::error::Error>> {
        let mut form_data = HashMap::new();
        form_data.insert("username".to_string(), "user@example.com".to_string());
        form_data.insert("password".to_string(), "secret123".to_string());

        let request = HttpRequest::post_form("/login?next=dashboard", form_data.clone());

        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/login");
        assert_eq!(request.query_params.get("next"), Some(&"dashboard".to_string()));
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/x-www-form-urlencoded".to_string()));
        assert!(request.headers.contains_key("Content-Length"));
        assert_eq!(request.form_data, form_data);

        Ok(())
    }
}
