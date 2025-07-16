//! URL encoding and form data utilities.

use std::collections::HashMap;

/// URL encode a string for safe transmission in URLs.
pub fn url_encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// URL decode a string, handling both %XX encoding and + for spaces.
pub fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '+' => result.push(' '),
            '%' => {
                let hex_str: String = chars.by_ref().take(2).collect();
                if hex_str.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte as char);
                    } else {
                        result.push('%');
                        result.push_str(&hex_str);
                    }
                } else {
                    result.push('%');
                    result.push_str(&hex_str);
                }
            }
            _ => result.push(ch),
        }
    }
    result
}

/// Parse a query string into a HashMap of parameters.
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if query.is_empty() {
        return params;
    }

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = url_decode(key);
            let decoded_value = url_decode(value);
            params.insert(decoded_key, decoded_value);
        } else {
            // Handle case where there's no '=' (treat as flag with empty value)
            let decoded_key = url_decode(pair);
            params.insert(decoded_key, String::new());
        }
    }
    params
}

/// Parse form-encoded data into a HashMap.
pub fn parse_form_string(form_data: &str) -> HashMap<String, String> {
    // Form data uses the same format as query strings
    parse_query_string(form_data)
}

/// Encode form data as application/x-www-form-urlencoded.
pub fn encode_form_data(form_data: &HashMap<String, String>) -> Vec<u8> {
    let mut pairs = Vec::new();
    for (key, value) in form_data {
        let encoded_key = url_encode(key);
        let encoded_value = url_encode(value);
        pairs.push(format!("{}={}", encoded_key, encoded_value));
    }
    pairs.join("&").into_bytes()
}

/// Build a query string from parameters.
pub fn build_query_string(params: &HashMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut pairs = Vec::new();
    for (key, value) in params {
        let encoded_key = url_encode(key);
        if value.is_empty() {
            pairs.push(encoded_key);
        } else {
            let encoded_value = url_encode(value);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
    }
    pairs.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding_decoding() {
        // Test URL encoding
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("test&demo"), "test%26demo");
        assert_eq!(url_encode("user@example.com"), "user%40example.com");

        // Test URL decoding
        assert_eq!(url_decode("hello+world"), "hello world");
        assert_eq!(url_decode("test%26demo"), "test&demo");
        assert_eq!(url_decode("user%40example.com"), "user@example.com");
    }

    #[test]
    fn test_form_parsing() {
        // Test parsing simple form data
        let form_data = parse_form_string("name=John&age=30");
        assert_eq!(form_data.get("name"), Some(&"John".to_string()));
        assert_eq!(form_data.get("age"), Some(&"30".to_string()));

        // Test URL encoded form data
        let form_data = parse_form_string("message=Hello%20World&special=%21%40%23");
        assert_eq!(form_data.get("message"), Some(&"Hello World".to_string()));
        assert_eq!(form_data.get("special"), Some(&"!@#".to_string()));
    }

    #[test]
    fn test_form_encoding() {
        let mut form_data = HashMap::new();
        form_data.insert("name".to_string(), "John Doe".to_string());
        form_data.insert("message".to_string(), "Hello, World!".to_string());

        let encoded = encode_form_data(&form_data);
        let encoded_str = String::from_utf8(encoded).unwrap();

        // Should contain properly encoded data
        assert!(encoded_str.contains("name=John+Doe"));
        assert!(encoded_str.contains("message=Hello%2C+World%21"));
    }
}
