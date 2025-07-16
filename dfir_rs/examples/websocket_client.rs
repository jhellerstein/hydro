use std::net::SocketAddr;

use dfir_rs::dfir_syntax;
use dfir_rs::util::{WebSocketMessage, WebSocketError};

#[tokio::main]
async fn main() {
    let server_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Creating WebSocket client to connect to {}", server_addr);
    
    // Create WebSocket client channels
    let (request_send, response_recv) = dfir_rs::util::connect_websocket_client();
    
    // Create DFIR flow for client operations
    let mut flow = dfir_syntax! {
        // Send test messages to server
        outgoing_messages = source_iter([
            WebSocketMessage::Text("Hello from DFIR WebSocket client!".to_string()),
            WebSocketMessage::Text("This is a test message".to_string()),
            WebSocketMessage::Ping(b"ping".to_vec()),
            WebSocketMessage::Binary(b"Binary data from client".to_vec()),
            WebSocketMessage::Text("Goodbye!".to_string()),
        ])
            -> enumerate()
            -> map(|(i, msg)| {
                println!("Sending message {}: {:?}", i, msg);
                (msg, server_addr)
            })
            -> dest_sink(request_send);

        // Receive messages from server
        incoming_messages = source_stream(response_recv)
            -> for_each(|result| {
                match result {
                    Ok((msg, addr)) => {
                        println!("Received from {}: {:?}", addr, msg);
                        match msg {
                            WebSocketMessage::Text(text) => {
                                println!("  Server says: {}", text);
                            }
                            WebSocketMessage::Binary(data) => {
                                println!("  Server sent binary data: {} bytes", data.len());
                            }
                            WebSocketMessage::Ping(data) => {
                                println!("  Server ping: {:?}", data);
                            }
                            WebSocketMessage::Pong(data) => {
                                println!("  Server pong: {:?}", data);
                            }
                            WebSocketMessage::Close(close_frame) => {
                                println!("  Server closing connection: {:?}", close_frame);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                    }
                }
            });
    };

    println!("Client flow running... Press Ctrl+C to stop");
    flow.run_available();
}
