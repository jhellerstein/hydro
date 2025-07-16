# DFIR WebSocket Chat Integration Demo

This document demonstrates the complete integration between the DFIR WebSocket Chat WASM web application and the native DFIR WebSocket chat server and clients.

## Architecture Overview

```
┌─────────────────┐    WebSocket    ┌─────────────────┐    WebSocket    ┌─────────────────┐
│   WASM Client   │◄─────────────── │  DFIR Server    │ ──────────────► │ Native Client   │
│  (Browser)      │                 │  (Rust)         │                 │  (Terminal)     │
│                 │                 │                 │                 │                 │
│ ┌─────────────┐ │                 │ ┌─────────────┐ │                 │ ┌─────────────┐ │
│ │ JavaScript  │ │                 │ │ DFIR Graph  │ │                 │ │ DFIR Graph  │ │
│ │ Frontend    │ │                 │ │ WebSocket   │ │                 │ │ WebSocket   │ │
│ │             │ │                 │ │ Server      │ │                 │ │ Client      │ │
│ └─────────────┘ │                 │ └─────────────┘ │                 │ └─────────────┘ │
│ ┌─────────────┐ │                 │                 │                 │                 │
│ │ WASM Module │ │                 │                 │                 │                 │
│ │ (Rust+DFIR) │ │                 │                 │                 │                 │
│ └─────────────┘ │                 │                 │                 │                 │
└─────────────────┘                 └─────────────────┘                 └─────────────────┘
```

## Message Flow

All clients (WASM and native) use the same message protocol:

```json
{
  "type": "chat",
  "content": "Hello, world!",
  "sender": "WebUser",
  "timestamp": 1642765432000.0
}
```

### Message Types

1. **Chat Messages**: Regular text messages between users
2. **User Events**: Join/leave notifications
3. **System Messages**: Server announcements
4. **User List**: Active users synchronization

## Running the Complete Demo

### Step 1: Start the DFIR WebSocket Server

```bash
cd /Users/jmh/code/hydro
cargo run -p dfir_rs --example chat_websocket -- \
    --name "ChatServer" \
    --role server \
    --address 127.0.0.1:3000
```

### Step 2: Start the Web Server

```bash
cd /Users/jmh/code/hydro/website_playground
python3 -m http.server 8080
```

### Step 3: Open WASM Web Client

Open browser to: `http://localhost:8080/examples/websocket_chat/`

- Server URL: `ws://localhost:3000`
- Username: `WebUser`
- Click "Connect"

### Step 4: Connect Native Client

```bash
cd /Users/jmh/code/hydro
cargo run -p dfir_rs --example chat_websocket -- \
    --name "TerminalUser" \
    --role client \
    --address 127.0.0.1:3000
```

### Step 5: Multi-Client Chat

Now you have:
- ✅ DFIR WebSocket Server (Rust native)
- ✅ WASM Web Client (Rust compiled to WebAssembly)  
- ✅ Native Terminal Client (Rust native)

All clients can chat with each other in real-time!

## Testing Scenarios

### Scenario 1: Cross-Platform Messaging
1. Send message from web browser → appears in terminal
2. Send message from terminal → appears in web browser
3. Multiple browser tabs → all receive messages

### Scenario 2: User Management
1. Web user joins → terminal sees "WebUser joined"
2. Terminal user leaves → web browser sees "TerminalUser left"
3. User list updates in real-time

### Scenario 3: Protocol Compatibility
1. Both clients use identical JSON message format
2. WebSocket frames handled consistently
3. Error handling works across platforms

## DFIR Architecture Benefits

### Server (Native Rust)
```rust
dfir_syntax! {
    // WebSocket connections
    connections = source_stream(websocket_stream);
    
    // Message routing
    connections -> demux(route_by_type) -> broadcast_to_all;
    
    // Real-time processing
    messages -> map(parse_json) -> filter(validate) -> for_each(send_to_clients);
}
```

### WASM Client (Rust → WebAssembly)
```rust
// Event-driven processing with futures
wasm_bindgen_futures::spawn_local(async move {
    while let Some(event) = event_receiver.next().await {
        match event {
            WebSocketEvent::Message(msg) => process_message(msg),
            WebSocketEvent::Connected => update_ui_connected(),
            // ... handle all events reactively
        }
    }
});
```

### Native Client (Rust)
```rust
dfir_syntax! {
    // Terminal input
    user_input = source_stream(stdin_reader);
    
    // WebSocket messages
    server_messages = source_stream(websocket_recv);
    
    // Bidirectional communication
    user_input -> map(format_message) -> dest_sink(websocket_send);
    server_messages -> map(parse_json) -> for_each(display_message);
}
```

## Key Features Demonstrated

### 1. **Protocol Interoperability**
- Same WebSocket protocol across all platforms
- Consistent JSON message format
- Compatible error handling

### 2. **DFIR Reactive Programming**
- Event-driven message processing
- Stream-based architecture
- Functional composition

### 3. **WebAssembly Integration**
- Rust code running in browser
- Native performance for message processing
- Seamless JavaScript interop

### 4. **Real-time Communication**
- Bidirectional WebSocket streams
- Low-latency message delivery
- Multi-client synchronization

### 5. **Cross-Platform Compatibility**
- Web browsers (via WASM)
- Terminal applications (native)
- Server applications (native)

## Performance Characteristics

### WASM Client
- **Advantages**: Near-native performance, sandboxed execution, broad browser support
- **Use Case**: Web applications, browser-based tools, client-side processing

### Native Client  
- **Advantages**: Full system access, maximum performance, advanced terminal features
- **Use Case**: Server administration, development tools, high-performance applications

### DFIR Server
- **Advantages**: High concurrency, reactive architecture, efficient resource usage
- **Use Case**: Real-time services, streaming applications, multi-client servers

## Extension Possibilities

### Additional Client Types
- Mobile apps (via React Native + WASM)
- Desktop applications (via Tauri + WASM)
- IoT devices (native Rust)

### Enhanced Features
- File sharing through WebSocket
- Voice/video integration with WebRTC
- Chat rooms and private messaging
- Message persistence and history

### Scalability
- Multiple server instances
- Load balancing with DFIR
- Distributed message routing

This demo showcases the power of DFIR's unified programming model across different deployment targets while maintaining consistent reactive programming patterns.
