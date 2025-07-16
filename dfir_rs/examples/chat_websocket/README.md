# WebSocket Chat Example

This example demonstrates a WebSocket-based chat server and client built with DFIR. The server can handle multiple clients, 
supports user names, and broadcasts messages to all connected clients.

## Features

- Multi-client WebSocket chat server
- User name support with `/name <username>` command
- Real-time message broadcasting
- Proper WebSocket protocol handling (ping/pong, close frames)
- DFIR streaming architecture for message processing
- Command-line interface for server and client roles

## Running the Example

To run the chat example, you'll need two terminals - one for the server and one (or more) for clients.

### Running the Server
In one terminal, start the chat server:
```shell
cargo run -p dfir_rs --example chat_websocket -- --name "Server" --role server
```

You can also specify a custom address:
```shell
cargo run -p dfir_rs --example chat_websocket -- --name "Server" --role server --address 127.0.0.1:9090
```

The server will start listening on `127.0.0.1:8080` by default.

### Running Clients
In another terminal (or multiple terminals for multiple clients), run the client:
```shell
cargo run -p dfir_rs --example chat_websocket -- --name "Alice" --role client
```

To connect to a custom server address:
```shell
cargo run -p dfir_rs --example chat_websocket -- --name "Alice" --role client --address 127.0.0.1:9090
```

### Alternative: Using wscat for Testing
You can also test the WebSocket chat server using `wscat`:
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

1. Start the server in one terminal:
   ```shell
   cargo run -p dfir_rs --example chat_websocket -- --name "Server" --role server
   ```

2. Connect with a client in another terminal:
   ```shell
   cargo run -p dfir_rs --example chat_websocket -- --name "Alice" --role client
   ```

3. The client will automatically set its name and join the chat
4. Type messages in the client terminal to send them to all connected clients

### Command Line Options

- `--name <name>`: Your display name (required)
- `--role <client|server>`: Whether to run as client or server (required)
- `--address <ip:port>`: Server address to bind to (server) or connect to (client) (optional, defaults to 127.0.0.1:8080)
- `--graph <type>`: Print a graph representation of the DFIR flow (optional)

## Architecture

The example uses DFIR's streaming operators to:
- Process incoming WebSocket messages from clients
- Handle different message types (Text, Ping, Close, etc.)
- Parse chat commands (like `/name`)
- Broadcast messages to all connected clients
- Maintain WebSocket connection state
