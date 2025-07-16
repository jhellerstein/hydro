//[imports]//
use std::net::SocketAddr;

use dfir_rs::{dfir_syntax, var_args};
use tokio::task::LocalSet;

//[/imports]//
use crate::Opts;

/// Run the HTTP server
pub async fn run_server(opts: &Opts) {
    run_http_server(opts.address).await.unwrap();
}

/// Example HTTP server using DFIR
///
/// This demonstrates how to create an HTTP server that:
/// 1. Binds to a local address
/// 2. Processes incoming HTTP requests through a DFIR pipeline
/// 3. Sends responses back to clients
///
/// Usage: cargo run --example http_server
async fn run_http_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    //[bind_server]//
    // HTTP operations require LocalSet - wrap TCP/HTTP operations in LocalSet
    LocalSet::new().run_until(async {
        // Bind HTTP server to localhost:3000
        let (response_send, request_recv, bound_addr) = dfir_rs::util::bind_http_server(addr).await;

        println!("🚀 HTTP Server listening on http://{}", bound_addr);
        println!("📋 Try these endpoints:");
        println!("   GET  http://{}/", bound_addr);
        println!("   GET  http://{}/api/hello", bound_addr);
        println!("   POST http://{}/api/echo (with JSON body)", bound_addr);
        println!("   GET  http://{}/404 (will return 404)", bound_addr);
        println!();
        println!("Press Ctrl+C to stop the server.");
        //[/bind_server]//

    let mut server = dfir_syntax! {
        //[route_handlers]//
        // Stream of incoming HTTP requests - use demux for efficient routing
        requests = source_stream(request_recv)
            -> map(|result: Result<(dfir_rs::util::HttpRequest, SocketAddr), dfir_rs::util::HttpCodecError>| {
                let (request, client_addr) = result.unwrap();
                println!("{} {} {} from {}",
                    request.version, request.method, request.path, client_addr);
                (request, client_addr)
            })
            -> demux(|(request, client_addr): (dfir_rs::util::HttpRequest, SocketAddr), var_args!(home, api_hello, api_echo, not_found)| {
                match request.path.as_str() {
                    "/" => home.give((request, client_addr)),
                    "/api/hello" => api_hello.give((request, client_addr)),
                    "/api/echo" => api_echo.give((request, client_addr)),
                    _ => not_found.give((request, client_addr)),
                }
            });
        //[/route_handlers]//

        //[response_union]//
        // Union all response streams together
        responses = union();

        // Handle home page requests
        requests[home]
            -> map(|(_request, client_addr)| {
                let response = dfir_rs::util::HttpResponse::ok()
                    .with_header("Content-Type", "text/html")
                    .with_body(b"<h1>Welcome to DFIR HTTP Server!</h1><p>Try visiting <a href=\"/api/hello\">/api/hello</a></p>".to_vec());
                (response, client_addr)
            })
            -> [0]responses;

        // Handle /api/hello requests
        requests[api_hello]
            -> map(|(_request, client_addr)| {
                let response = dfir_rs::util::HttpResponse::ok()
                    .with_header("Content-Type", "application/json")
                    .with_body(b"{\"message\": \"Hello from DFIR!\", \"version\": \"HTTP/1.1\"}".to_vec());
                (response, client_addr)
            })
            -> [1]responses;

        // Handle /api/echo requests
        requests[api_echo]
            -> map(|(request, client_addr)| {
                let body = String::from_utf8_lossy(&request.body);
                let echo_response = format!("{{\"echoed\": {}}}", body);
                let response = dfir_rs::util::HttpResponse::ok()
                    .with_header("Content-Type", "application/json")
                    .with_body(echo_response.into_bytes());
                (response, client_addr)
            })
            -> [2]responses;

        // Handle all other requests (404)
        requests[not_found]
            -> map(|(request, client_addr)| {
                let not_found_body = format!("{{\"error\": \"Not Found\", \"path\": \"{}\"}}", request.path);
                let response = dfir_rs::util::HttpResponse::not_found()
                    .with_header("Content-Type", "application/json")
                    .with_body(not_found_body.into_bytes());
                (response, client_addr)
            })
            -> [3]responses;

        // Send responses back to clients
        responses -> dest_sink(response_send);
        //[/response_union]//
    };

    //[run_server]//
    server.run_async().await.unwrap();
    //[/run_server]//
    }).await;
    Ok(())
}
