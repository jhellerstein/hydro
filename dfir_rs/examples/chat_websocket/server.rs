//[imports]//
use dfir_rs::dfir_syntax;
use dfir_rs::util::WebSocketMessage;

use crate::{Opts, default_server_address};
//[/imports]//

pub(crate) async fn run_server(opts: Opts) {
    // Use the server address that was provided in the command-line arguments, or use the default
    // if one was not provided.
    let addr = opts.address.unwrap_or_else(default_server_address);
    
    println!("Starting WebSocket chat server on {}", addr);
    
    //[bind_websocket]//
    // Create WebSocket server channels
    let (response_send, request_recv, addr) = dfir_rs::util::bind_websocket_server(addr).await;
    println!("Chat server bound to: {}", addr);
    //[/bind_websocket]//
    
    //[dfir_flow]//
    // Create DFIR flow to handle chat messages
    let mut flow = dfir_syntax! {
        // Receive messages from clients
        input_stream = source_stream(request_recv)
            -> map(|result| {
                match result {
                    Ok((msg, client_addr)) => {
                        println!("Received from {}: {:?}", client_addr, msg);
                        Some((msg, client_addr))
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
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
    //[/dfir_flow]//

    println!("Chat server running... Press Ctrl+C to stop");
    flow.run_async().await;
}
