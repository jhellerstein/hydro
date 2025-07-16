/// Example demonstrating HTTP query parameter parsing in DFIR
///
/// This example shows how to:
/// 1. Parse query parameters from HTTP requests
/// 2. Use helper methods to access parameters
/// 3. Build responses based on query parameters
/// 4. Use status code helpers for proper HTTP responses
#[dfir_rs::main]
async fn main() {
    // Example 1: Basic query parameter parsing
    let search_request =
        dfir_rs::util::HttpRequest::get("/api/search?q=rust+programming&limit=10&format=json");

    println!("🔍 Search Request Example:");
    println!("   Path: {}", search_request.path);
    println!("   Query Parameters:");
    for (key, value) in &search_request.query_params {
        println!("     {} = {}", key, value);
    }

    // Example 2: Using helper methods
    println!("\n🛠️  Helper Methods Example:");
    if let Some(query) = search_request.get_query_param("q") {
        println!("   Search query: {}", query);
    }

    if let Some(limit) = search_request.get_query_param("limit") {
        println!("   Results limit: {}", limit);
    }

    println!(
        "   Has format param: {}",
        search_request.has_query_param("format")
    );
    println!(
        "   Has page param: {}",
        search_request.has_query_param("page")
    );

    // Example 3: Building requests with query parameters
    let api_request = dfir_rs::util::HttpRequest::get("/api/users")
        .with_query_param("id", "123")
        .with_query_param("include", "profile")
        .with_query_param("debug", ""); // Flag-style parameter

    println!("\n🏗️  Built Request Example:");
    println!("   Full URL: {}", api_request.full_url());

    // Example 4: URL encoding in action
    let encoded_request = dfir_rs::util::HttpRequest::get("/search")
        .with_query_param("q", "hello world & special chars")
        .with_query_param("email", "user@example.com");

    println!("\n🔒 URL Encoding Example:");
    println!("   Full URL: {}", encoded_request.full_url());

    // Example 5: Simulating a search API response based on parameters
    let response = if search_request.has_query_param("format")
        && search_request.get_query_param("format") == Some(&"json".to_string())
    {
        // Return JSON response
        let search_results = serde_json::json!({
            "query": search_request.get_query_param("q").unwrap_or(&"".to_string()),
            "limit": search_request.get_query_param("limit").unwrap_or(&"20".to_string()),
            "results": [
                {"title": "Rust Programming Language", "url": "https://rust-lang.org"},
                {"title": "Rust Book", "url": "https://doc.rust-lang.org/book/"}
            ]
        });

        dfir_rs::util::HttpResponse::ok()
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_vec(&search_results).unwrap())
    } else {
        // Return HTML response
        let html = format!(
            "<html><body><h1>Search Results</h1><p>Query: {}</p></body></html>",
            search_request
                .get_query_param("q")
                .unwrap_or(&"".to_string())
        );

        dfir_rs::util::HttpResponse::ok()
            .with_header("Content-Type", "text/html")
            .with_body(html.into_bytes())
    };

    println!("\n📄 Response Example:");
    println!("   Status: {}", response.status_code);
    println!(
        "   Content-Type: {}",
        response
            .headers
            .get("Content-Type")
            .unwrap_or(&"none".to_string())
    );
    println!("   Body size: {} bytes", response.body.len());

    println!("\n✅ Query parameter parsing enables rich web APIs!");
    println!("💡 Use query parameters for filtering, pagination, format selection, and more!");

    // Example 6: Demonstrating HTTP status code helpers
    println!("\n🎯 HTTP Status Code Helpers Example:");

    // Success responses
    let ok_response = dfir_rs::util::HttpResponse::ok().with_body(b"Success!".to_vec());
    let created_response = dfir_rs::util::HttpResponse::created()
        .with_header("Location", "/api/users/123")
        .with_body(b"User created".to_vec());
    let no_content_response = dfir_rs::util::HttpResponse::no_content();

    println!(
        "   ✅ Success responses: {} OK, {} Created, {} No Content",
        ok_response.status_code, created_response.status_code, no_content_response.status_code
    );

    // Redirect responses
    let redirect_response = dfir_rs::util::HttpResponse::found("/new-location");
    let permanent_redirect =
        dfir_rs::util::HttpResponse::moved_permanently("https://example.com/moved");
    let not_modified_response = dfir_rs::util::HttpResponse::not_modified();

    println!(
        "   🔄 Redirect responses: {} Found, {} Moved Permanently, {} Not Modified",
        redirect_response.status_code,
        permanent_redirect.status_code,
        not_modified_response.status_code
    );

    // Client error responses
    let bad_request = dfir_rs::util::HttpResponse::bad_request();
    let unauthorized = dfir_rs::util::HttpResponse::unauthorized();
    let forbidden = dfir_rs::util::HttpResponse::forbidden();
    let not_found = dfir_rs::util::HttpResponse::not_found();
    let method_not_allowed = dfir_rs::util::HttpResponse::method_not_allowed();
    let conflict = dfir_rs::util::HttpResponse::conflict();
    let unprocessable = dfir_rs::util::HttpResponse::unprocessable_entity();

    println!(
        "   ❌ Client errors: {} Bad Request, {} Unauthorized, {} Forbidden, {} Not Found",
        bad_request.status_code,
        unauthorized.status_code,
        forbidden.status_code,
        not_found.status_code
    );
    println!(
        "   ❌ More client errors: {} Method Not Allowed, {} Conflict, {} Unprocessable Entity",
        method_not_allowed.status_code, conflict.status_code, unprocessable.status_code
    );

    // Server error responses
    let server_error = dfir_rs::util::HttpResponse::internal_server_error();
    let bad_gateway = dfir_rs::util::HttpResponse::bad_gateway();
    let service_unavailable = dfir_rs::util::HttpResponse::service_unavailable();

    println!(
        "   🔥 Server errors: {} Internal Server Error, {} Bad Gateway, {} Service Unavailable",
        server_error.status_code, bad_gateway.status_code, service_unavailable.status_code
    );

    // Example of rich error handling with query parameters
    let error_response = if search_request.has_query_param("format")
        && search_request.get_query_param("format") == Some(&"json".to_string())
    {
        dfir_rs::util::HttpResponse::bad_request()
            .with_header("Content-Type", "application/json")
            .with_body(
                serde_json::to_vec(&serde_json::json!({
                    "error": "Missing required parameter",
                    "code": "MISSING_PARAM",
                    "details": "The 'q' parameter is required for search operations"
                }))
                .unwrap(),
            )
    } else {
        dfir_rs::util::HttpResponse::bad_request()
            .with_header("Content-Type", "text/html")
            .with_body(b"<html><body><h1>400 Bad Request</h1><p>Missing required parameter 'q'</p></body></html>".to_vec())
    };

    println!("\n🛠️  Rich error handling with format-aware responses!");
    println!(
        "   Status: {}, Content-Type: {}",
        error_response.status_code,
        error_response
            .headers
            .get("Content-Type")
            .unwrap_or(&"none".to_string())
    );
}
