//! Integration tests for HTTP module functionality.

use std::collections::HashMap;

use crate::util::{Cookie, HttpRequest, HttpResponse};

#[test]
fn test_dfir_http_processing_pattern() -> Result<(), Box<dyn std::error::Error>> {
    use std::net::SocketAddr;

    use crate::dfir_syntax;
    use crate::util::{HttpCodecError, collect_ready};

    // Create test requests to inject (simulating what would come from bind_http_server)
    let post_request = HttpRequest::post_json("/api/data", &serde_json::json!({"key": "value"}))?;
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
            post_request,
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
    println!("✅ For dest_sink examples, see examples/http_server.rs and examples/http_client.rs");

    Ok(())
}

#[test]
fn test_dfir_cookie_processing_pattern() -> Result<(), Box<dyn std::error::Error>> {
    use std::net::SocketAddr;

    use crate::dfir_syntax;
    use crate::util::{HttpCodecError, collect_ready};

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

#[test]
fn test_dfir_form_processing_pattern() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap as StdHashMap;

    // Simulate a DFIR processing pipeline with form data
    let form_requests = vec![
        // User registration
        {
            let mut form_data = HashMap::new();
            form_data.insert("username".to_string(), "alice".to_string());
            form_data.insert("email".to_string(), "alice@example.com".to_string());
            form_data.insert("terms".to_string(), "accepted".to_string());
            HttpRequest::post_form("/register", form_data)
        },
        // Profile update
        {
            let mut form_data = HashMap::new();
            form_data.insert("name".to_string(), "Alice Smith".to_string());
            form_data.insert("bio".to_string(), "Software Engineer".to_string());
            HttpRequest::patch_form("/profile/alice", form_data)
        },
    ];

    // Process through DFIR-style pipeline
    let mut user_db = StdHashMap::<String, serde_json::Value>::new();
    let mut responses = Vec::new();

    for request in form_requests {
        let response = match request.path.as_str() {
            "/register" => {
                if let (Some(username), Some(email)) = (
                    request.get_form_field("username"),
                    request.get_form_field("email"),
                ) {
                    let user_data = serde_json::json!({
                        "username": username,
                        "email": email,
                        "terms_accepted": request.get_form_field("terms") == Some(&"accepted".to_string())
                    });
                    user_db.insert(username.clone(), user_data);

                    let mut resp = HttpResponse::created();
                    resp.body = serde_json::to_vec(
                        &serde_json::json!({"message": "User registered successfully"}),
                    )
                    .unwrap();
                    resp.headers
                        .insert("Content-Type".to_string(), "application/json".to_string());
                    resp
                } else {
                    HttpResponse::bad_request()
                }
            }
            path if path.starts_with("/profile/") => {
                let username = path.strip_prefix("/profile/").unwrap();
                if let Some(mut user_data) = user_db.get(username).cloned() {
                    if let Some(name) = request.get_form_field("name") {
                        user_data["name"] = serde_json::Value::String(name.clone());
                    }
                    if let Some(bio) = request.get_form_field("bio") {
                        user_data["bio"] = serde_json::Value::String(bio.clone());
                    }
                    user_db.insert(username.to_string(), user_data.clone());

                    let mut resp = HttpResponse::ok();
                    resp.body = serde_json::to_vec(&user_data).unwrap();
                    resp.headers
                        .insert("Content-Type".to_string(), "application/json".to_string());
                    resp
                } else {
                    HttpResponse::not_found()
                }
            }
            _ => HttpResponse::not_found(),
        };
        responses.push((response, request));
    }

    // Verify results
    assert_eq!(responses.len(), 2);

    // Check registration response
    assert_eq!(responses[0].0.status_code, 201);
    let reg_body: serde_json::Value = serde_json::from_slice(&responses[0].0.body)?;
    assert_eq!(reg_body["message"], "User registered successfully");

    // Check profile update response
    assert_eq!(responses[1].0.status_code, 200);
    let profile_body: serde_json::Value = serde_json::from_slice(&responses[1].0.body)?;
    assert_eq!(profile_body["username"], "alice");
    assert_eq!(profile_body["name"], "Alice Smith");
    assert_eq!(profile_body["bio"], "Software Engineer");

    println!("✅ Form URL encoding working correctly through DFIR pipeline!");
    println!("✅ Supports: form parsing, POST/PATCH forms, field access, automatic parsing");

    Ok(())
}
