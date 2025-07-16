# WebSocket Chat Example

This example demonstrates a WebSocket-based chat server and client built with DFIR. The server can handle multiple clients, 
supports user names, and broadcasts messages to all connected clients.

## Features

- Multi-client WebSocket chat server
- User name support with `/name <username>` command
- Real-time message broadcasting
- Proper WebSocket protocol handling (ping/pong, close frames)
- DFIR streaming architecture for message processing

## Running the Example

To run the chat example, you'll need two terminals - one for the server and one (or more) for clients.

### Running the Server
In one terminal, start the chat server:
```shell
cargo run -p dfir_rs --example chat_websocket
```

The server will start listening on `127.0.0.1:8080` by default.

### Running Clients
In another terminal (or multiple terminals for multiple clients), run the client:
```shell
cargo run -p dfir_rs --bin chat_websocket --bin client
```

Note: If the client binary doesn't work, you can also use a WebSocket client tool like `wscat`:
```shell
# Install wscat if you don't have it
npm install -g wscat

# Connect to the chat server
wscat -c ws://127.0.0.1:8080
```

### Chat Commands

Once connected, you can use these commands:
- `/name <your_name>` - Set your display name in the chat
- Type any other message to send it to all connected clients
- The server will automatically handle ping/pong for connection keepalive

### Example Session

1. Start the server in one terminal
2. Connect with a client in another terminal  
3. Set your name: `/name Alice`
4. Send messages: `Hello everyone!`
5. Messages will be broadcast to all connected clients with your name prefix

## Architecture

The example uses DFIR's streaming operators to:
- Process incoming WebSocket messages from clients
- Handle different message types (Text, Ping, Close, etc.)
- Parse chat commands (like `/name`)
- Broadcast messages to all connected clients
- Maintain WebSocket connection state
