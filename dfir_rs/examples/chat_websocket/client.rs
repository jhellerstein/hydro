use std::net::SocketAddr;
use std::io::{self, BufRead};

use dfir_rs::dfir_syntax;
use dfir_rs::util::WebSocketMessage;

#[tokio::main]
async fn main() {
    let server_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Connecting to WebSocket chat server at {}", server_addr);
    println!("Commands:");
    println!("  /name <your_name> - Set your name");
    println!("  Type any message to chat");
    println!("  Ctrl+C to exit");
    
    // Create WebSocket client channels
    let (request_send, response_recv) = dfir_rs::util::connect_websocket_client();
    
    // Create DFIR flow for client operations
    let mut flow = dfir_syntax! {
        // Send join message to server
        initial_message = source_iter([
            WebSocketMessage::Text("/name Guest".to_string()),
            WebSocketMessage::Text("Hello everyone! I just joined the chat.".to_string()),
        ])
            -> map(|msg| {
                println!("Sending: {:?}", msg);
                (msg, server_addr)
            })
            -> dest_sink(request_send);

        // Receive messages from server
        incoming_messages = source_stream(response_recv)
            -> for_each(|result| {
                match result {
                    Ok((msg, _addr)) => {
                        match msg {
                            WebSocketMessage::Text(text) => {
                                println!("[CHAT] {}", text);
                            }
                            WebSocketMessage::Ping(data) => {
                                println!("[PING] Server sent ping: {:?}", data);
                            }
                            WebSocketMessage::Pong(data) => {
                                println!("[PONG] Server sent pong: {:?}", data);
                            }
                            WebSocketMessage::Close(close_frame) => {
                                println!("[CLOSE] Server closing connection: {:?}", close_frame);
                            }
                            WebSocketMessage::Binary(_) => {
                                println!("[BINARY] Server sent binary data (not supported in chat)");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                    }
                }
            });
    };

    println!("Chat client running... Press Ctrl+C to stop");
    flow.run_available();
}
