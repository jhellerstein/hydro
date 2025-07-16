//! Cookie handling for HTTP requests and responses.

use std::collections::HashMap;
use crate::util::http::types::{Cookie, SameSite};

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

    /// Set the domain for this cookie.
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the path for this cookie.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the max age for this cookie (in seconds).
    pub fn with_max_age(mut self, max_age: i64) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the expires attribute for this cookie.
    pub fn with_expires(mut self, expires: impl Into<String>) -> Self {
        self.expires = Some(expires.into());
        self
    }

    /// Mark this cookie as secure (HTTPS only).
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Mark this cookie as HTTP only (not accessible via JavaScript).
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set the SameSite attribute for this cookie.
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Generate the Set-Cookie header value for this cookie.
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
            header.push_str(&format!("; SameSite={}", same_site));
        }

        header
    }

    /// Parse a Cookie header value into a HashMap of cookie name-value pairs.
    ///
    /// Example: "session_id=abc123; user_pref=dark_mode"
    pub fn parse_cookie_header(header: &str) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        for pair in header.split(';') {
            let pair = pair.trim();
            if let Some((name, value)) = pair.split_once('=') {
                let name = name.trim().to_string();
                let value = value.trim().to_string();
                cookies.insert(name, value);
            }
            // Ignore malformed entries (no '=' sign)
        }

        cookies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
