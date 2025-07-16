# DFIR WebSocket Chat (WASM)

A real-time chat application built with DFIR (Dataflow IR) compiled to WebAssembly, demonstrating how to use DFIR's reactive programming model in web browsers with WebSocket connections.

## Overview

This example showcases:

- **DFIR in WASM**: Using DFIR's reactive dataflow programming model in the browser
- **WebSocket Integration**: Connecting to WebSocket servers using the browser's native WebSocket API
- **Real-time Chat**: Multi-user chat with join/leave notifications and user lists
- **Stream Processing**: Processing WebSocket events through DFIR's stream operators
- **Error Handling**: Robust error handling with user feedback
- **State Management**: Managing chat state through DFIR flows

## Architecture

The application consists of three main components:

1. **HTML/JavaScript Frontend** (`index.html`): User interface and WASM integration
2. **Rust WASM Module** (`websocket_chat.rs`): DFIR-based WebSocket chat logic
3. **WebSocket Server**: Compatible with the existing DFIR WebSocket chat server

### DFIR Integration

The WebSocket chat uses DFIR's reactive programming model:

```rust
dfir_syntax! {
    // Source of WebSocket events
    events = source_stream(event_receiver);
    
    // Demux events by type
    events -> demux(|event, var_args!(connected, disconnected, messages, errors, send_requests)| {
        match event {
            WebSocketEvent::Connected => connected.give(()),
            WebSocketEvent::Disconnected => disconnected.give(()),
            WebSocketEvent::Message(msg) => messages.give(msg),
            WebSocketEvent::Error(err) => errors.give(err),
            WebSocketEvent::SendMessage(msg) => send_requests.give(msg),
        }
    });
    
    // Process each event type through separate flows
    connected -> for_each(|_| { /* handle connection */ });
    messages -> map(parse_json) -> for_each(|chat_msg| { /* handle messages */ });
    send_requests -> for_each(|content| { /* send messages */ });
}
```

### Message Types

The chat supports these message types:

```rust
enum ChatMessage {
    Chat { content: String, sender: String, timestamp: f64 },
    UserJoined { username: String, timestamp: f64 },
    UserLeft { username: String, timestamp: f64 },
    UserList { users: Vec<String> },
    System { content: String, timestamp: f64 },
}
```

## Building and Running

### Prerequisites

1. **Rust with WASM target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **wasm-bindgen CLI**:
   ```bash
   cargo install wasm-bindgen-cli
   ```

3. **WebSocket Server**: Run the DFIR WebSocket chat server:
   ```bash
   cd ../../../dfir_rs/examples/chat_websocket
   cargo run -- --role server --address 127.0.0.1:3000
   ```

### Build WASM Module

From the `website_playground` directory:

```bash
# Build the WASM module
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-bindgen --out-dir pkg --target web target/wasm32-unknown-unknown/release/website_playground.wasm
```

### Serve the Web Application

You need to serve the files over HTTP (not file://) due to WASM security restrictions:

```bash
# Option 1: Python HTTP server
python3 -m http.server 8081

# Option 2: Node.js http-server
npx http-server -p 8081

# Option 3: Any other local HTTP server
```

Then open: `http://localhost:8081/examples/websocket_chat/`

## Usage

1. **Start WebSocket Server**: First, run the DFIR WebSocket chat server (see above)

2. **Open Web App**: Navigate to the chat application in your browser

3. **Connect**: 
   - Enter server URL (default: `ws://localhost:3000`)
   - Enter your username
   - Click "Connect"

4. **Chat**:
   - Type messages and press Enter or click Send
   - See other users join/leave
   - View active user list
   - Receive real-time messages

## Features

### Real-time Communication
- Bidirectional WebSocket communication
- Instant message delivery
- Connection status indicators
- Automatic reconnection handling

### User Experience
- Clean, responsive UI
- Message timestamps
- User identification
- Active user list
- Error notifications
- Connection status feedback

### DFIR Integration
- Stream-based event processing
- Reactive state management
- Functional message transformation
- Error handling through demux
- Asynchronous execution with futures

### Browser Compatibility
- Uses native WebSocket API
- WASM for high-performance Rust code
- Modern JavaScript async/await
- Responsive CSS design

## Message Flow

1. **User Types Message** → JavaScript event → WASM function call
2. **WASM Processing** → DFIR stream → Event demux → Message formatting
3. **WebSocket Send** → Browser WebSocket API → Server
4. **Server Broadcast** → All connected clients (including WASM client)
5. **Receive Processing** → DFIR stream → JSON parsing → UI update

## Error Handling

The application handles various error scenarios:

- **Connection failures**: Network issues, server unavailable
- **Message errors**: Invalid JSON, send failures
- **Protocol errors**: Unexpected message formats
- **WebSocket errors**: Connection drops, protocol violations

All errors are processed through DFIR's error handling patterns and displayed to users.

## Development

### Adding New Message Types

1. Add to `ChatMessage` enum in `websocket_chat.rs`
2. Update message parsing in DFIR graph
3. Add UI handling in JavaScript
4. Update server compatibility

### Extending DFIR Functionality

The DFIR graph can be extended with additional stream processing:

```rust
// Add message filtering
messages -> filter(|msg| !msg.content.is_empty()) -> /* existing processing */

// Add message transformation  
messages -> map(|msg| enhance_message(msg)) -> /* existing processing */

// Add custom operators
messages -> custom_operator() -> /* existing processing */
```

### Performance Optimization

- Use `--release` builds for production
- Optimize WASM size with `wee_alloc`
- Bundle JavaScript for production
- Enable gzip compression on server

## Integration with DFIR Server

This client is compatible with the DFIR WebSocket chat server:

```bash
# Server
cargo run --example chat_websocket -- --role server

# Client (this WASM app)
# Connect via browser to ws://localhost:3000
```

The server and client use the same message protocol, enabling seamless communication between Rust native clients and WASM web clients.

## Future Enhancements

- **File sharing**: Upload and share files through WebSocket
- **Rich text**: Markdown support, emoji, formatting
- **Chat rooms**: Multiple channels, private messaging
- **Authentication**: User login, secure connections
- **Persistence**: Message history, offline support
- **Voice/Video**: WebRTC integration for multimedia chat

## Learning Resources

- [DFIR Documentation](../../../docs/docs/dfir/)
- [WebSocket Protocol RFC 6455](https://tools.ietf.org/html/rfc6455)
- [WebAssembly Documentation](https://webassembly.org/docs/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [web-sys Documentation](https://docs.rs/web-sys/)

This example demonstrates the power of combining DFIR's reactive programming model with modern web technologies to create responsive, real-time applications.
