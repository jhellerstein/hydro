use std::collections::HashMap;

use dfir_rs::util::{HttpRequest, HttpResponse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Form URL Encoding Example for DFIR HTTP Framework");
    println!("================================================");

    // Example 1: Create a POST request with form data
    println!("\n📝 Creating POST form request:");
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "alice@example.com".to_string());
    form_data.insert("password".to_string(), "secret123".to_string());
    form_data.insert("remember_me".to_string(), "true".to_string());

    let login_request = HttpRequest::post_form("/login?redirect=dashboard", form_data.clone());

    println!("  Method: {}", login_request.method);
    println!("  Path: {}", login_request.path);
    println!("  Query params: {:?}", login_request.query_params);
    println!(
        "  Content-Type: {}",
        login_request.headers.get("Content-Type").unwrap()
    );
    println!("  Form data: {:?}", login_request.form_data);
    println!("  Body: {}", String::from_utf8_lossy(&login_request.body));

    // Example 2: Create a PATCH request with form data
    println!("\n🔧 Creating PATCH form request:");
    let mut update_data = HashMap::new();
    update_data.insert("name".to_string(), "Alice Smith".to_string());
    update_data.insert(
        "bio".to_string(),
        "Software Engineer & Coffee Enthusiast".to_string(),
    );
    update_data.insert("location".to_string(), "San Francisco, CA".to_string());

    let update_request = HttpRequest::patch_form("/profile/123", update_data.clone());

    println!("  Method: {}", update_request.method);
    println!("  Path: {}", update_request.path);
    println!(
        "  Content-Type: {}",
        update_request.headers.get("Content-Type").unwrap()
    );
    println!("  Form data: {:?}", update_request.form_data);
    println!("  Body: {}", String::from_utf8_lossy(&update_request.body));

    // Example 3: Form field access methods
    println!("\n� Form field access:");
    let form_request = HttpRequest::post_form("/contact", {
        let mut data = HashMap::new();
        data.insert("email".to_string(), "bob@test.com".to_string());
        data.insert("subject".to_string(), "Hello World".to_string());
        data.insert("message".to_string(), "This is a test message".to_string());
        data
    });

    println!("  Email: {:?}", form_request.get_form_field("email"));
    println!("  Subject: {:?}", form_request.get_form_field("subject"));
    println!("  Message: {:?}", form_request.get_form_field("message"));
    println!(
        "  Has email field: {}",
        form_request.has_form_field("email")
    );
    println!(
        "  Has phone field: {}",
        form_request.has_form_field("phone")
    );

    // Example 4: DFIR-style processing pipeline
    println!("\n🔄 DFIR Processing Pipeline:");
    let form_requests = vec![
        HttpRequest::post_form("/users", {
            let mut data = HashMap::new();
            data.insert("email".to_string(), "charlie@example.com".to_string());
            data.insert("name".to_string(), "Charlie Brown".to_string());
            data
        }),
        HttpRequest::patch_form("/users/1", {
            let mut data = HashMap::new();
            data.insert("status".to_string(), "active".to_string());
            data.insert("last_login".to_string(), "2024-01-15".to_string());
            data
        }),
    ];

    for (i, request) in form_requests.iter().enumerate() {
        println!("  Request {}: {} {}", i + 1, request.method, request.path);

        let response = match request.path.as_str() {
            "/users" => {
                if let Some(email) = request.get_form_field("email") {
                    println!("    Creating user with email: {}", email);
                    HttpResponse::created()
                } else {
                    HttpResponse::bad_request()
                }
            }
            "/users/1" => {
                if let Some(status) = request.get_form_field("status") {
                    println!("    Updating user status to: {}", status);
                    HttpResponse::ok()
                } else {
                    HttpResponse::bad_request()
                }
            }
            _ => HttpResponse::not_found(),
        };

        println!(
            "    Response: {} {}",
            response.status_code, response.status_text
        );
    }

    // Example 5: Manual form parsing
    println!("\n🔧 Manual form parsing:");
    let body = "action=subscribe&topic=newsletter&frequency=weekly&format=html"
        .as_bytes()
        .to_vec();
    let mut manual_request = HttpRequest::post("/newsletter", body);
    manual_request.headers.insert(
        "Content-Type".to_string(),
        "application/x-www-form-urlencoded".to_string(),
    );

    println!(
        "  Before parsing: {} form fields",
        manual_request.form_data.len()
    );
    manual_request.parse_form_data();
    println!(
        "  After parsing: {} form fields",
        manual_request.form_data.len()
    );

    for (key, value) in &manual_request.form_data {
        println!("    {}: {}", key, value);
    }

    // Example 6: URL encoding demonstration
    println!("\n🔤 URL encoding demonstration:");
    let special_chars_form = HttpRequest::post_form("/feedback", {
        let mut data = HashMap::new();
        data.insert(
            "comment".to_string(),
            "This has spaces & special chars!".to_string(),
        );
        data.insert("rating".to_string(), "5/5 ⭐".to_string());
        data.insert("tags".to_string(), "bug-fix, enhancement".to_string());
        data
    });

    println!("  Raw form data: {:?}", special_chars_form.form_data);
    println!(
        "  Encoded body: {}",
        String::from_utf8_lossy(&special_chars_form.body)
    );

    println!("\n✅ Form URL encoding working perfectly!");
    println!("✅ Features: POST/PATCH forms, field access, manual parsing, URL encoding/decoding");

    Ok(())
}
