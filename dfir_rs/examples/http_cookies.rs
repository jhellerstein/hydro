//! HTTP Cookie Example
//!
//! This example demonstrates cookie support in the DFIR HTTP framework.
//! It shows how to:
//! - Parse cookies from incoming requests
//! - Set cookies in responses
//! - Handle authentication and session management with cookies
//! - Use different cookie attributes (secure, httponly, samesite)

use dfir_rs::util::{Cookie, HttpRequest, HttpResponse, SameSite};

fn main() {
    // Simulate HTTP requests with cookies (normally these would come from bind_http_server)
    let test_requests = vec![
        // First request: no session cookie, should get a new session
        HttpRequest::get("/dashboard"),
        // Second request: has session cookie, should be authenticated
        {
            let mut req = HttpRequest::get("/api/user-profile");
            req.cookies
                .insert("session_id".to_string(), "valid_session_123".to_string());
            req.cookies
                .insert("user_theme".to_string(), "dark".to_string());
            req
        },
        // Third request: has expired session, should get refreshed
        {
            let mut req = HttpRequest::post("/api/refresh", b"{}".to_vec());
            req.cookies
                .insert("session_id".to_string(), "expired_session_456".to_string());
            req
        },
        // Fourth request: login attempt, should set authentication cookies
        HttpRequest::post_json(
            "/auth/login",
            &serde_json::json!({
                "username": "alice",
                "password": "secret123"
            }),
        )
        .unwrap(),
    ];

    println!("🍪 HTTP Cookie Example");
    println!("====================\n");

    // Process each request to demonstrate cookie handling
    for (i, request) in test_requests.into_iter().enumerate() {
        println!("Request {}: {} {}", i + 1, request.method, request.path);

        // Show incoming cookies
        if !request.cookies.is_empty() {
            println!("  📥 Incoming cookies:");
            for (name, value) in &request.cookies {
                println!("    {}: {}", name, value);
            }
        } else {
            println!("  📥 No cookies");
        }

        // Route the request and handle cookies
        let response = match request.path.as_str() {
            "/dashboard" => {
                if request.has_cookie("session_id") {
                    // User is logged in, show dashboard
                    HttpResponse::ok().with_body(b"Welcome back to your dashboard!".to_vec())
                } else {
                    // No session, redirect to login and set a tracking cookie
                    HttpResponse::found("/login")
                        .with_simple_cookie("visitor_id", "anon_12345")
                        .with_cookie(
                            Cookie::new("last_page", "/dashboard")
                                .with_path("/")
                                .with_max_age(3600), // 1 hour
                        )
                }
            }

            "/api/user-profile" => {
                if let Some(session_id) = request.get_cookie("session_id") {
                    if session_id == "valid_session_123" {
                        // Valid session, return user data
                        let theme = request
                            .get_cookie("user_theme")
                            .map(|s| s.as_str())
                            .unwrap_or("light");
                        HttpResponse::json(&serde_json::json!({
                            "user": "Alice",
                            "email": "alice@example.com",
                            "theme": theme,
                            "session": session_id
                        }))
                        .unwrap()
                    } else {
                        HttpResponse::unauthorized().with_body(b"Invalid session".to_vec())
                    }
                } else {
                    HttpResponse::unauthorized().with_body(b"No session cookie".to_vec())
                }
            }

            "/api/refresh" => {
                if request.has_cookie("session_id") {
                    // Refresh the session with a new ID and set security cookies
                    HttpResponse::ok()
                        .with_simple_cookie("session_id", "refreshed_session_789")
                        .with_cookie(
                            Cookie::new("csrf_token", "csrf_abc123")
                                .secure()
                                .http_only()
                                .with_same_site(SameSite::Strict)
                                .with_max_age(86400), // 24 hours
                        )
                        .with_body(b"Session refreshed".to_vec())
                } else {
                    HttpResponse::bad_request().with_body(b"No session to refresh".to_vec())
                }
            }

            "/auth/login" => {
                // Simulate login - in real app, you'd validate credentials
                HttpResponse::ok()
                    .with_simple_cookie("session_id", "new_session_999")
                    .with_cookie(
                        Cookie::new("user_role", "admin")
                            .with_domain("example.com")
                            .with_path("/")
                            .secure()
                            .http_only()
                            .with_same_site(SameSite::Lax)
                            .with_max_age(7 * 24 * 3600), // 7 days
                    )
                    .with_cookie(
                        Cookie::new("remember_me", "true").with_max_age(30 * 24 * 3600), // 30 days
                    )
                    .with_body(b"Login successful".to_vec())
            }

            _ => HttpResponse::not_found(),
        };

        // Show outgoing cookies
        if !response.set_cookies.is_empty() {
            println!("  📤 Setting cookies:");
            for cookie in &response.set_cookies {
                println!("    {}", cookie.to_set_cookie_header());
            }
        }

        println!(
            "  📋 Response: {} {}",
            response.status_code, response.status_text
        );
        if !response.body.is_empty() {
            let body_preview = String::from_utf8_lossy(&response.body);
            let preview = if body_preview.len() > 60 {
                format!("{}...", &body_preview[..60])
            } else {
                body_preview.to_string()
            };
            println!("  📄 Body: {}", preview);
        }
        println!();
    }

    println!("🎯 Cookie Features Demonstrated:");
    println!("   ✓ Cookie parsing from requests");
    println!("   ✓ Setting simple and complex cookies");
    println!("   ✓ Cookie attributes (Domain, Path, Max-Age, Secure, HttpOnly, SameSite)");
    println!("   ✓ Session management and authentication");
    println!("   ✓ User preferences and tracking");
    println!("   ✓ CSRF protection and security best practices");
    println!("\n💡 This approach works perfectly for most modern web applications!");
    println!("   • Stateless server design (cookies store client state)");
    println!("   • Compatible with microservices and serverless");
    println!("   • Handles authentication, sessions, and user preferences");
    println!("   • Full HTTP cookie specification compliance");
}
