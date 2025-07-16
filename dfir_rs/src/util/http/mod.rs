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
//! - **Form URL encoding**: Full application/x-www-form-urlencoded support with automatic parsing
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
//! ## Form URL Encoding Support
//!
//! Full support for application/x-www-form-urlencoded:
//!
//! - **Automatic parsing**: Form data is automatically parsed from request bodies
//! - **Convenience methods**: `post_form()` and `patch_form()` for easy form creation
//! - **Field access**: `get_form_field()` and `has_form_field()` methods
//! - **URL encoding**: Proper encoding/decoding of special characters
//!
//! ```rust,no_run
//! use std::collections::HashMap;
//! use dfir_rs::util::{Cookie, HttpRequest, HttpResponse, SameSite};
//!
//! // Reading cookies from requests
//! let request = HttpRequest::get("/dashboard");
//! if let Some(session_id) = request.get_cookie("session_id") {
//!     println!("User session: {}", session_id);
//! }
//!
//! // Form data handling
//! let mut form_data = HashMap::new();
//! form_data.insert("username".to_string(), "alice".to_string());
//! form_data.insert("password".to_string(), "secret".to_string());
//! let form_req = HttpRequest::post_form("/login", form_data);
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
//! flow.run_available();
//! # }
//! ```

pub mod types;
pub mod encoding;
pub mod cookie;
pub mod request;
pub mod response;
pub mod codec;

#[cfg(test)]
/// Integration tests for HTTP functionality.
pub mod tests;

// Re-export the main types for backward compatibility
pub use types::{HttpRequest, HttpResponse, HttpCodecError, Cookie, SameSite};
pub use codec::{HttpCodec, HttpServerCodec, HttpClientCodec};

// Import external dependencies needed by the implementations
use serde::{Serialize, Deserialize};

// Implement Serialize/Deserialize for compatibility with existing code
impl Serialize for HttpRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("HttpRequest", 8)?;
        state.serialize_field("method", &self.method)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("headers", &self.headers)?;
        state.serialize_field("body", &self.body)?;
        state.serialize_field("query_params", &self.query_params)?;
        state.serialize_field("cookies", &self.cookies)?;
        state.serialize_field("form_data", &self.form_data)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for HttpRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Method,
            Path,
            Version,
            Headers,
            Body,
            QueryParams,
            Cookies,
            FormData,
        }

        struct HttpRequestVisitor;

        impl<'de> Visitor<'de> for HttpRequestVisitor {
            type Value = HttpRequest;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct HttpRequest")
            }

            fn visit_map<V>(self, mut map: V) -> Result<HttpRequest, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut method = None;
                let mut path = None;
                let mut version = None;
                let mut headers = None;
                let mut body = None;
                let mut query_params = None;
                let mut cookies = None;
                let mut form_data = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Method => {
                            if method.is_some() {
                                return Err(de::Error::duplicate_field("method"));
                            }
                            method = Some(map.next_value()?);
                        }
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::Headers => {
                            if headers.is_some() {
                                return Err(de::Error::duplicate_field("headers"));
                            }
                            headers = Some(map.next_value()?);
                        }
                        Field::Body => {
                            if body.is_some() {
                                return Err(de::Error::duplicate_field("body"));
                            }
                            body = Some(map.next_value()?);
                        }
                        Field::QueryParams => {
                            if query_params.is_some() {
                                return Err(de::Error::duplicate_field("query_params"));
                            }
                            query_params = Some(map.next_value()?);
                        }
                        Field::Cookies => {
                            if cookies.is_some() {
                                return Err(de::Error::duplicate_field("cookies"));
                            }
                            cookies = Some(map.next_value()?);
                        }
                        Field::FormData => {
                            if form_data.is_some() {
                                return Err(de::Error::duplicate_field("form_data"));
                            }
                            form_data = Some(map.next_value()?);
                        }
                    }
                }

                let method = method.ok_or_else(|| de::Error::missing_field("method"))?;
                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let headers = headers.ok_or_else(|| de::Error::missing_field("headers"))?;
                let body = body.ok_or_else(|| de::Error::missing_field("body"))?;
                let query_params = query_params.unwrap_or_default();
                let cookies = cookies.unwrap_or_default();
                let form_data = form_data.unwrap_or_default();

                Ok(HttpRequest {
                    method,
                    path,
                    version,
                    headers,
                    body,
                    query_params,
                    cookies,
                    form_data,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["method", "path", "version", "headers", "body", "query_params", "cookies", "form_data"];
        deserializer.deserialize_struct("HttpRequest", FIELDS, HttpRequestVisitor)
    }
}

impl Serialize for HttpResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("HttpResponse", 6)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("status_code", &self.status_code)?;
        state.serialize_field("status_text", &self.status_text)?;
        state.serialize_field("headers", &self.headers)?;
        state.serialize_field("set_cookies", &self.set_cookies)?;
        state.serialize_field("body", &self.body)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for HttpResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Version,
            StatusCode,
            StatusText,
            Headers,
            SetCookies,
            Body,
        }

        struct HttpResponseVisitor;

        impl<'de> Visitor<'de> for HttpResponseVisitor {
            type Value = HttpResponse;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct HttpResponse")
            }

            fn visit_map<V>(self, mut map: V) -> Result<HttpResponse, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut version = None;
                let mut status_code = None;
                let mut status_text = None;
                let mut headers = None;
                let mut set_cookies = None;
                let mut body = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::StatusCode => {
                            if status_code.is_some() {
                                return Err(de::Error::duplicate_field("status_code"));
                            }
                            status_code = Some(map.next_value()?);
                        }
                        Field::StatusText => {
                            if status_text.is_some() {
                                return Err(de::Error::duplicate_field("status_text"));
                            }
                            status_text = Some(map.next_value()?);
                        }
                        Field::Headers => {
                            if headers.is_some() {
                                return Err(de::Error::duplicate_field("headers"));
                            }
                            headers = Some(map.next_value()?);
                        }
                        Field::SetCookies => {
                            if set_cookies.is_some() {
                                return Err(de::Error::duplicate_field("set_cookies"));
                            }
                            set_cookies = Some(map.next_value()?);
                        }
                        Field::Body => {
                            if body.is_some() {
                                return Err(de::Error::duplicate_field("body"));
                            }
                            body = Some(map.next_value()?);
                        }
                    }
                }

                let version = version.ok_or_else(|| de::Error::missing_field("version"))?;
                let status_code = status_code.ok_or_else(|| de::Error::missing_field("status_code"))?;
                let status_text = status_text.ok_or_else(|| de::Error::missing_field("status_text"))?;
                let headers = headers.ok_or_else(|| de::Error::missing_field("headers"))?;
                let set_cookies = set_cookies.unwrap_or_default();
                let body = body.ok_or_else(|| de::Error::missing_field("body"))?;

                Ok(HttpResponse {
                    version,
                    status_code,
                    status_text,
                    headers,
                    set_cookies,
                    body,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["version", "status_code", "status_text", "headers", "set_cookies", "body"];
        deserializer.deserialize_struct("HttpResponse", FIELDS, HttpResponseVisitor)
    }
}

impl Serialize for Cookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Cookie", 9)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("domain", &self.domain)?;
        state.serialize_field("path", &self.path)?;
        state.serialize_field("max_age", &self.max_age)?;
        state.serialize_field("expires", &self.expires)?;
        state.serialize_field("secure", &self.secure)?;
        state.serialize_field("http_only", &self.http_only)?;
        state.serialize_field("same_site", &self.same_site)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Cookie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Name,
            Value,
            Domain,
            Path,
            MaxAge,
            Expires,
            Secure,
            HttpOnly,
            SameSite,
        }

        struct CookieVisitor;

        impl<'de> Visitor<'de> for CookieVisitor {
            type Value = Cookie;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Cookie")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Cookie, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut value = None;
                let mut domain = None;
                let mut path = None;
                let mut max_age = None;
                let mut expires = None;
                let mut secure = None;
                let mut http_only = None;
                let mut same_site = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Value => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value()?);
                        }
                        Field::Domain => {
                            if domain.is_some() {
                                return Err(de::Error::duplicate_field("domain"));
                            }
                            domain = Some(map.next_value()?);
                        }
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::MaxAge => {
                            if max_age.is_some() {
                                return Err(de::Error::duplicate_field("max_age"));
                            }
                            max_age = Some(map.next_value()?);
                        }
                        Field::Expires => {
                            if expires.is_some() {
                                return Err(de::Error::duplicate_field("expires"));
                            }
                            expires = Some(map.next_value()?);
                        }
                        Field::Secure => {
                            if secure.is_some() {
                                return Err(de::Error::duplicate_field("secure"));
                            }
                            secure = Some(map.next_value()?);
                        }
                        Field::HttpOnly => {
                            if http_only.is_some() {
                                return Err(de::Error::duplicate_field("http_only"));
                            }
                            http_only = Some(map.next_value()?);
                        }
                        Field::SameSite => {
                            if same_site.is_some() {
                                return Err(de::Error::duplicate_field("same_site"));
                            }
                            same_site = Some(map.next_value()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;
                let domain = domain;
                let path = path;
                let max_age = max_age;
                let expires = expires;
                let secure = secure.unwrap_or(false);
                let http_only = http_only.unwrap_or(false);
                let same_site = same_site;

                Ok(Cookie {
                    name,
                    value,
                    domain,
                    path,
                    max_age,
                    expires,
                    secure,
                    http_only,
                    same_site,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["name", "value", "domain", "path", "max_age", "expires", "secure", "http_only", "same_site"];
        deserializer.deserialize_struct("Cookie", FIELDS, CookieVisitor)
    }
}
