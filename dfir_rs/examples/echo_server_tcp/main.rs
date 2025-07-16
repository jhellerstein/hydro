//[imports]//
use std::net::SocketAddr;
use bytes::{Bytes, BytesMut};
use tokio::task::LocalSet;

use dfir_rs::{dfir_syntax, var_args};
use dfir_rs::util::bind_tcp_bytes;
//[/imports]//

/// Example TCP echo server using DFIR
///
/// This demonstrates how to create a TCP server that:
/// 1. Binds to a local address and accepts TCP connections
/// 2. Processes incoming TCP messages through a DFIR pipeline
/// 3. Echoes messages back to clients over the same TCP connection
///
/// Usage: cargo run --example tcp_echo_server
///
/// To test the server, you can use telnet or netcat:
/// ```bash
/// telnet localhost 3001
/// # or
/// nc localhost 3001
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //[bind_tcp]//
    // Bind TCP server to localhost:3001
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    
    // TCP operations require LocalSet for testing/example purposes
    let local = LocalSet::new();
    let (response_send, request_recv, bound_addr) = local.run_until(async {
        bind_tcp_bytes(addr).await
    }).await;
    //[/bind_tcp]//

    println!("🚀 TCP Echo Server listening on {}", bound_addr);
    println!("📋 Test with: telnet {} {}", bound_addr.ip(), bound_addr.port());
    println!("📋 Or with: nc {} {}", bound_addr.ip(), bound_addr.port());
    println!("📋 Type messages and they will be echoed back!");
    println!();
    println!("Press Ctrl+C to stop the server.");

    //[dfir_flow]//
    let mut server = dfir_syntax! {
        //[request_processing]//
        // Stream of incoming TCP messages - use demux for error handling
        messages = source_stream(request_recv)
            -> demux(|result: Result<(BytesMut, SocketAddr), std::io::Error>, var_args!(success, error)| {
                match result {
                    Ok((data, client_addr)) => {
                        println!("📨 Received {} bytes from {}", data.len(), client_addr);
                        let message = String::from_utf8_lossy(&data);
                        println!("   Content: {:?}", message.trim());
                        success.give((data, client_addr))
                    },
                    Err(err) => {
                        println!("❌ Error receiving data: {:?}", err);
                        error.give(err)
                    },
                }
            });
        //[/request_processing]//

        //[echo_logic]//
        // Echo messages back to clients
        messages[success]
            -> map(|(data, client_addr): (BytesMut, SocketAddr)| {
                // Create echo response by prepending "Echo: " to the original message
                let echo_message = format!("Echo: {}", String::from_utf8_lossy(&data));
                println!("📤 Echoing to {}: {:?}", client_addr, echo_message.trim());
                (Bytes::from(echo_message.into_bytes()), client_addr)
            })
            -> dest_sink(response_send);
        //[/echo_logic]//

        //[error_handling]//
        // Handle connection errors (optional - for debugging)
        messages[error]
            -> for_each(|err| {
                eprintln!("🔥 TCP error: {}", err);
            });
        //[/error_handling]//
    };
    //[/dfir_flow]//

    //[run_server]//
    println!("🎯 TCP Echo Server is running...");
    local.run_until(async {
        server.run_async().await
    }).await;
    //[/run_server]//
    
    Ok(())
}
