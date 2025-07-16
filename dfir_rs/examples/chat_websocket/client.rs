//[imports]//
use dfir_rs::dfir_syntax;
use dfir_rs::util::WebSocketMessage;

use crate::{Opts, default_server_address};
//[/imports]//

pub(crate) async fn run_client(opts: Opts) {
    // Use the server address that was provided in the command-line arguments, or use the default
    // if one was not provided.
    let server_addr = opts.address.unwrap_or_else(default_server_address);
    
    println!("Connecting to WebSocket chat server at {}", server_addr);
    println!("Your name: {}", opts.name);
    println!("Commands:");
    println!("  /name <your_name> - Set your name");
    println!("  Type any message to chat");
    println!("  Ctrl+C to exit");
    
    //[connect_websocket]//
    // Create WebSocket client channels
    let (request_send, response_recv) = dfir_rs::util::connect_websocket_client();
    //[/connect_websocket]//
    
    //[dfir_flow]//
    // Create DFIR flow for client operations
    let mut flow = dfir_syntax! {
        // Send initial name setting and join message
        initial_messages = source_iter([
            WebSocketMessage::Text(format!("/name {}", opts.name)),
            WebSocketMessage::Text(format!("{} joined the chat!", opts.name)),
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
    //[/dfir_flow]//

    println!("Chat client running... Press Ctrl+C to stop");
    flow.run_async().await;
}
