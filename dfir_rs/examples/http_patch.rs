//! HTTP PATCH Method Example
//!
//! This example demonstrates PATCH method support in the DFIR HTTP framework.
//! PATCH is commonly used in REST APIs for partial resource updates.
//!
//! PATCH vs PUT:
//! - PUT: Replace the entire resource
//! - PATCH: Apply partial modifications to a resource

use dfir_rs::util::{HttpRequest, HttpResponse};

fn main() {
    println!("🔧 HTTP PATCH Method Example");
    println!("============================\n");

    // Example 1: PATCH with JSON - Partial user profile update
    println!("📝 Example 1: User Profile Update");
    let user_update = serde_json::json!({
        "email": "alice.new@example.com",
        "preferences": {
            "theme": "dark",
            "notifications": true
        }
    });

    let patch_req = HttpRequest::patch_json("/api/users/123", &user_update)
        .expect("Should create PATCH request");

    println!("  Method: {}", patch_req.method);
    println!("  Path: {}", patch_req.path);
    println!(
        "  Content-Type: {}",
        patch_req.headers.get("Content-Type").unwrap()
    );
    println!("  Body: {}", String::from_utf8_lossy(&patch_req.body));

    // Simulate server processing
    let response = HttpResponse::ok()
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"id": 123, "email": "alice.new@example.com", "updated": true}"#.to_vec());

    println!(
        "  Response: {} {}\n",
        response.status_code, response.status_text
    );

    // Example 2: PATCH with raw data - Update specific fields
    println!("📝 Example 2: Product Inventory Update");
    let inventory_patch = br#"{"quantity": 150, "last_updated": "2025-07-15T10:30:00Z"}"#;

    let patch_req = HttpRequest::patch("/api/products/456?audit=true", inventory_patch.to_vec())
        .with_header("Content-Type", "application/json");

    println!("  Method: {}", patch_req.method);
    println!("  Path: {}", patch_req.path);
    println!("  Query Params: {:?}", patch_req.query_params);
    println!("  Body: {}", String::from_utf8_lossy(&patch_req.body));

    let response = HttpResponse::ok()
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"id": 456, "quantity": 150, "status": "updated"}"#.to_vec());

    println!(
        "  Response: {} {}\n",
        response.status_code, response.status_text
    );

    // Example 3: JSON Patch - RFC 6902 format
    println!("📝 Example 3: JSON Patch (RFC 6902) Operations");
    let json_patch_ops = serde_json::json!([
        {
            "op": "replace",
            "path": "/status",
            "value": "active"
        },
        {
            "op": "add",
            "path": "/tags/-",
            "value": "featured"
        },
        {
            "op": "remove",
            "path": "/deprecated_field"
        }
    ]);

    let patch_req = HttpRequest::patch_json("/api/articles/789", &json_patch_ops)
        .expect("Should create JSON Patch request")
        .with_header("Content-Type", "application/json-patch+json");

    println!("  Method: {}", patch_req.method);
    println!("  Path: {}", patch_req.path);
    println!(
        "  Content-Type: {}",
        patch_req.headers.get("Content-Type").unwrap()
    );
    println!("  JSON Patch Operations:");
    let parsed: serde_json::Value = serde_json::from_slice(&patch_req.body).unwrap();
    for (i, op) in parsed.as_array().unwrap().iter().enumerate() {
        println!("    {}: {} {}", i + 1, op["op"], op["path"]);
    }

    let response = HttpResponse::ok()
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"id": 789, "status": "active", "operations_applied": 3}"#.to_vec());

    println!(
        "  Response: {} {}\n",
        response.status_code, response.status_text
    );

    // Example 4: Error handling - Validation failure
    println!("📝 Example 4: PATCH Error Handling");
    let invalid_patch = serde_json::json!({
        "email": "invalid-email",  // Invalid email format
        "age": -5                  // Invalid age
    });

    let patch_req = HttpRequest::patch_json("/api/users/999", &invalid_patch)
        .expect("Should create PATCH request");

    println!("  Method: {}", patch_req.method);
    println!("  Path: {}", patch_req.path);

    // Simulate validation error response
    let error_response = HttpResponse::unprocessable_entity()
        .with_header("Content-Type", "application/json")
        .with_body(br#"{"error": "Validation failed", "details": ["Invalid email format", "Age must be positive"]}"#.to_vec());

    println!(
        "  Response: {} {}",
        error_response.status_code, error_response.status_text
    );
    println!(
        "  Error Body: {}\n",
        String::from_utf8_lossy(&error_response.body)
    );

    println!("🎯 PATCH Method Use Cases:");
    println!("   ✓ User profile updates (email, preferences)");
    println!("   ✓ Product inventory changes (quantity, price)");
    println!("   ✓ JSON Patch operations (RFC 6902)");
    println!("   ✓ Status updates (active/inactive, published/draft)");
    println!("   ✓ Partial document modifications");
    println!("   ✓ Configuration updates");

    println!("\n💡 PATCH Best Practices:");
    println!("   • Use for partial updates (not full replacement)");
    println!("   • Include only changed fields in request body");
    println!("   • Return updated resource or confirmation");
    println!("   • Handle validation errors gracefully");
    println!("   • Consider idempotency for critical operations");
    println!("   • Use appropriate Content-Type (application/json or application/json-patch+json)");
}
