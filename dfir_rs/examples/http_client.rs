use std::net::SocketAddr;

use dfir_rs::{dfir_syntax, var_args};

/// Example HTTP client using DFIR
///
/// This demonstrates how to create an HTTP client that:
/// 1. Sends HTTP requests to a server
/// 2. Processes responses through a DFIR pipeline
///
/// Make sure to run the http_server example first:
/// cargo run --example http_server
///
/// Then run this client:
/// cargo run --example http_client
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the HTTP server (assumes server is running on localhost:3000)
    let server_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let (request_send, response_recv) = dfir_rs::util::connect_http_client();

    println!("🔗 HTTP Client connecting to http://{}", server_addr);

    let mut client = dfir_syntax! {
        // Create a series of HTTP requests and send them
        source_iter([
            dfir_rs::util::HttpRequest::get("/"),
            dfir_rs::util::HttpRequest::get("/api/hello"),
            dfir_rs::util::HttpRequest::post_json("/api/echo", &serde_json::json!({
                "message": "Hello from DFIR client!",
                "timestamp": "2025-07-15T12:00:00Z"
            })).unwrap(),
            dfir_rs::util::HttpRequest::get("/404"),
        ])
        -> map(|request| (request, server_addr))
        -> dest_sink(request_send);

        // Process responses using demux for clean separation
        responses = source_stream(response_recv) -> demux(|result, var_args!(success, error)| {
            match result {
                Ok((response, addr)) => success.give((response, addr)),
                Err(err) => error.give(err),
            }
        });

        // Handle successful responses
        responses[success]
            -> for_each(|(response, _addr): (dfir_rs::util::HttpResponse, SocketAddr)| {
                println!("\n📨 Response received:");
                println!("   Status: {} {}", response.status_code, response.status_text);
                println!("   Headers: {:?}", response.headers);

                if let Some(content_type) = response.headers.get("Content-Type") {
                    if content_type.contains("application/json") {
                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&response.body) {
                            println!("   Body (JSON): {}", serde_json::to_string_pretty(&json).unwrap());
                        } else {
                            println!("   Body: {}", String::from_utf8_lossy(&response.body));
                        }
                    } else {
                        let body_preview = String::from_utf8_lossy(&response.body);
                        let preview = if body_preview.len() > 100 {
                            format!("{}...", &body_preview[..100])
                        } else {
                            body_preview.to_string()
                        };
                        println!("   Body: {}", preview);
                    }
                } else {
                    println!("   Body: {}", String::from_utf8_lossy(&response.body));
                }
            });

        // Handle errors
        responses[error]
            -> for_each(|err| {
                println!("❌ Error: {:?}", err);
            });
    };

    // Run the client
    // Use timeout to allow the client to finish after processing responses
    match tokio::time::timeout(std::time::Duration::from_secs(5), client.run_async()).await {
        Ok(_) => println!("\n✅ Client completed successfully"),
        Err(_) => println!("\n⏰ Client finished processing requests and responses"),
    }

    Ok(())
}
