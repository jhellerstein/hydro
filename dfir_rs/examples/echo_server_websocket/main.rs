use std::net::SocketAddr;

use dfir_rs::dfir_syntax;
use dfir_rs::util::WebSocketMessage;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("Starting WebSocket echo server on {}", addr);
    
    // Create WebSocket server channels
    let (response_send, request_recv, addr) = dfir_rs::util::bind_websocket_server(addr).await;
    
    println!("WebSocket server bound to: {}", addr);
    
    // Create DFIR flow to handle WebSocket messages
    let mut flow = dfir_syntax! {
        input_stream = source_stream(request_recv)
            -> map(|x| {
                match x {
                    Ok((msg, client_addr)) => {
                        println!("Received message from {}: {:?}", client_addr, msg);
                        Some((msg, client_addr))
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
                        None
                    }
                }
            })
            -> filter_map(|x| x);

        // Echo messages back to clients
        echo_stream = input_stream
            -> map(|(msg, addr)| {
                match msg {
                    WebSocketMessage::Text(text) => {
                        println!("Echoing text: {}", text);
                        (WebSocketMessage::Text(format!("Echo: {}", text)), addr)
                    }
                    WebSocketMessage::Binary(data) => {
                        println!("Echoing binary data ({} bytes)", data.len());
                        (WebSocketMessage::Binary(data), addr)
                    }
                    WebSocketMessage::Ping(data) => {
                        println!("Responding to ping with pong");
                        (WebSocketMessage::Pong(data), addr)
                    }
                    WebSocketMessage::Pong(_) => {
                        println!("Received pong");
                        return (WebSocketMessage::Text("Pong received".to_string()), addr);
                    }
                    WebSocketMessage::Close(close_frame) => {
                        println!("Client closing connection: {:?}", close_frame);
                        (WebSocketMessage::Close(close_frame), addr)
                    }
                }
            })
            -> dest_sink(response_send);
    };

    println!("Echo server running... Press Ctrl+C to stop");
    flow.run_available();
}
