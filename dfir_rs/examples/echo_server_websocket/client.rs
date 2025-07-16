use std::net::SocketAddr;

use dfir_rs::dfir_syntax;
use dfir_rs::util::WebSocketMessage;

#[tokio::main]
async fn main() {
    let server_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Creating WebSocket client to test echo server at {}", server_addr);
    
    // Create WebSocket client channels
    let (request_send, response_recv) = dfir_rs::util::connect_websocket_client();
    
    // Create DFIR flow for client operations
    let mut flow = dfir_syntax! {
        // Send test messages to echo server
        outgoing_messages = source_iter([
            WebSocketMessage::Text("Hello echo server!".to_string()),
            WebSocketMessage::Text("This is a test message".to_string()),
            WebSocketMessage::Ping(b"ping test".to_vec()),
            WebSocketMessage::Binary(b"Binary data from client".to_vec()),
            WebSocketMessage::Text("Final message".to_string()),
        ])
            -> enumerate()
            -> map(|(i, msg)| {
                println!("Sending message {}: {:?}", i + 1, msg);
                (msg, server_addr)
            })
            -> dest_sink(request_send);

        // Receive echoed messages from server
        incoming_messages = source_stream(response_recv)
            -> for_each(|result| {
                match result {
                    Ok((msg, addr)) => {
                        match msg {
                            WebSocketMessage::Text(text) => {
                                println!("Echo received: {}", text);
                            }
                            WebSocketMessage::Binary(data) => {
                                println!("Binary echo received: {} bytes", data.len());
                            }
                            WebSocketMessage::Ping(data) => {
                                println!("Server ping: {:?}", data);
                            }
                            WebSocketMessage::Pong(data) => {
                                println!("Server pong (response to our ping): {:?}", data);
                            }
                            WebSocketMessage::Close(close_frame) => {
                                println!("Server closing connection: {:?}", close_frame);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error receiving message: {:?}", e);
                    }
                }
            });
    };

    println!("Echo client running... Press Ctrl+C to stop");
    flow.run_available();
}
