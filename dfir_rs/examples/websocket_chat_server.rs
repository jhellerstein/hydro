use std::net::SocketAddr;

use dfir_rs::dfir_syntax;
use dfir_rs::util::{WebSocketMessage, WebSocketError};
use tokio::task::LocalSet;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Starting WebSocket chat server on {}", addr);
    
    // WebSocket operations require LocalSet for proper execution
    LocalSet::new().run_until(async {
        println!("Creating WebSocket server channels...");
        // Create WebSocket server channels
        let (response_send, request_recv, addr) = dfir_rs::util::bind_websocket_server(addr).await;
    println!("Chat server bound to: {}", addr);
    println!("WebSocket server ready to accept connections");
    
    // Create DFIR flow to handle chat messages
    let mut flow = dfir_syntax! {
        // Receive messages from clients
        input_stream = source_stream(request_recv)
            -> map(|result| {
                match result {
                    Ok((msg, client_addr)) => {
                        println!("Successfully received from {}: {:?}", client_addr, msg);
                        Some((msg, client_addr))
                    }
                    Err(e) => {
                        eprintln!("WebSocket error from client: {:?}", e);
                        // Log more details about the error
                        match &e {
                            dfir_rs::util::WebSocketError::InvalidFrame(details) => {
                                eprintln!("Frame parsing error details: {}", details);
                            }
                            dfir_rs::util::WebSocketError::Io(io_err) => {
                                eprintln!("IO error: {}", io_err);
                            }
                            dfir_rs::util::WebSocketError::InvalidUtf8(utf8_err) => {
                                eprintln!("UTF8 error: {}", utf8_err);
                            }
                            dfir_rs::util::WebSocketError::HandshakeError(handshake_err) => {
                                eprintln!("Handshake error: {}", handshake_err);
                            }
                            dfir_rs::util::WebSocketError::ProtocolViolation(protocol_err) => {
                                eprintln!("Protocol violation: {}", protocol_err);
                            }
                            dfir_rs::util::WebSocketError::FrameTooLarge { size, max_size } => {
                                eprintln!("Frame too large: {} bytes (max: {})", size, max_size);
                            }
                            dfir_rs::util::WebSocketError::ConnectionClosed { code, reason } => {
                                eprintln!("Connection closed: {:?} - {}", code, reason);
                            }
                        }
                        None
                    }
                }
            })
            -> filter_map(|x| x);

        // Handle different message types
        chat_messages = input_stream
            -> map(|(msg, client_addr)| {
                match msg {
                    WebSocketMessage::Text(text) => {
                        if text.starts_with("/name ") {
                            // User setting their name
                            let name = text[6..].trim().to_string();
                            println!("Client {} set name to: {}", client_addr, name);
                            let welcome_msg = format!("Welcome, {}! You are now in the chat.", name);
                            (WebSocketMessage::Text(welcome_msg), client_addr)
                        } else if text.starts_with("/") {
                            // Other commands
                            let help_msg = "Unknown command. Available commands: /name <your_name>".to_string();
                            (WebSocketMessage::Text(help_msg), client_addr)
                        } else {
                            // Regular chat message
                            let chat_msg = format!("{}: {}", client_addr, text);
                            println!("Broadcasting: {}", chat_msg);
                            (WebSocketMessage::Text(chat_msg), client_addr)
                        }
                    }
                    WebSocketMessage::Ping(data) => {
                        println!("Ping from {}, sending pong", client_addr);
                        (WebSocketMessage::Pong(data), client_addr)
                    }
                    WebSocketMessage::Close(close_frame) => {
                        println!("Client {} disconnected: {:?}", client_addr, close_frame);
                        let leave_msg = format!("{} left the chat", client_addr);
                        (WebSocketMessage::Text(leave_msg), client_addr)
                    }
                    WebSocketMessage::Binary(_) => {
                        let error_msg = "Binary messages not supported in chat".to_string();
                        (WebSocketMessage::Text(error_msg), client_addr)
                    }
                    WebSocketMessage::Pong(_) => {
                        println!("Received pong from {}", client_addr);
                        let status_msg = "Pong received".to_string();
                        (WebSocketMessage::Text(status_msg), client_addr)
                    }
                }
            })
            -> dest_sink(response_send);
    };

    println!("Chat server running... Press Ctrl+C to stop");
    flow.run_async().await;
    }).await;
}
