#![cfg(not(target_arch = "wasm32"))]

//! WebSocket support for DFIR.
//!
//! This module provides WebSocket client and server support for real-time bidirectional
//! communication over TCP connections. It's designed to work seamlessly with DFIR's
//! streaming model for building real-time applications.
//!
//! ## Features
//!
//! - **RFC 6455 compliance**: Full WebSocket protocol implementation
//! - **Bidirectional streaming**: Perfect fit for DFIR's dataflow model
//! - **HTTP upgrade handshake**: Leverages existing HTTP infrastructure
//! - **Multiple message types**: Text, Binary, Ping, Pong, and Close frames
//! - **Connection management**: Automatic connection state handling
//! - **Error handling**: Comprehensive error types for different failure modes
//! - **Real-time**: Low-latency streaming for interactive applications
//!
//! ## Message Types
//!
//! WebSocket supports several frame types for different use cases:
//!
//! - **Text**: UTF-8 encoded string messages
//! - **Binary**: Raw binary data for efficient data transfer
//! - **Ping/Pong**: Keep-alive and latency measurement
//! - **Close**: Graceful connection termination with optional reason
//!
//! ## Use Cases
//!
//! WebSocket is ideal for real-time applications:
//!
//! - **Chat Applications**: Real-time messaging and notifications
//! - **Live Data Feeds**: Financial tickers, monitoring dashboards
//! - **Gaming**: Real-time multiplayer game networking
//! - **IoT Communication**: Device-to-server streaming data
//! - **Collaborative Tools**: Real-time document editing and collaboration
//!
//! ```rust,no_run
//! use dfir_rs::util::{WebSocketMessage, bind_websocket_server};
//! use dfir_rs::{dfir_syntax, var_args};
//! use tokio::task::LocalSet;
//!
//! # async fn example() {
//! // WebSocket operations require LocalSet for proper execution
//! let local = LocalSet::new();
//! let (response_send, request_recv, addr) = local
//!     .run_until(bind_websocket_server("127.0.0.1:8080".parse().unwrap()))
//!     .await;
//!
//! // Define your DFIR graph for real-time message processing
//! let mut flow = dfir_syntax! {
//!     messages = source_stream(request_recv)
//!       -> map(|result| result.unwrap())  // Handle errors appropriately in real code
//!       -> demux(|(message, client_addr), var_args!(text, binary, ping)| {
//!           match message {
//!               WebSocketMessage::Text(text) => text.give((text, client_addr)),
//!               WebSocketMessage::Binary(data) => binary.give((data, client_addr)),
//!               WebSocketMessage::Ping(data) => ping.give((data, client_addr)),
//!               _ => {}, // Handle other message types
//!           }
//!       });
//!
//!     // Union responses before sending
//!     responses = union() -> dest_sink(response_send);
//!
//!     // Echo text messages
//!     messages[text] -> map(|(text, client_addr)| {
//!         (WebSocketMessage::Text(format!("Echo: {}", text)), client_addr)
//!     }) -> responses;
//!
//!     // Echo binary messages
//!     messages[binary] -> map(|(data, client_addr)| {
//!         (WebSocketMessage::Binary(data), client_addr)
//!     }) -> responses;
//!
//!     // Respond to pings with pongs
//!     messages[ping] -> map(|(data, client_addr)| {
//!         (WebSocketMessage::Pong(data), client_addr)
//!     }) -> responses;
//! };
//!
//! // Run the real-time flow
//! flow.run_available();
//! # }
//! ```
//!
//! # Basic Usage
//!
//! ## WebSocket Server
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
//! use dfir_rs::util::{WebSocketMessage, bind_websocket_server};
//! use dfir_rs::dfir_syntax;
//! use tokio::task::LocalSet;
//!
//! # async fn example() {
//! let local = LocalSet::new();
//! let (response_send, request_recv, addr) = local
//!     .run_until(bind_websocket_server("127.0.0.1:8080".parse().unwrap()))
//!     .await;
//!
//! let mut flow = dfir_syntax! {
//!     source_stream(request_recv)
//!       -> map(|result| result.unwrap())
//!       -> map(|(message, client_addr)| {
//!           // Echo all messages back to the client
//!           (message, client_addr)
//!       })
//!       -> dest_sink(response_send);
//! };
//!
//! flow.run_available();
//! # }
//! ```

pub mod types;
pub mod codec;
pub mod handshake;
pub mod connection;

#[cfg(test)]
/// Integration tests for WebSocket functionality.
pub mod tests;

// Re-export the main types for backward compatibility
pub use types::{WebSocketMessage, WebSocketFrame, WebSocketOpcode, WebSocketError, WebSocketCloseCode};
pub use codec::{WebSocketCodec, WebSocketServerCodec, WebSocketClientCodec};
pub use handshake::{WebSocketHandshake, HandshakeRequest, HandshakeResponse};
pub use connection::{WebSocketConnection, ConnectionState};
