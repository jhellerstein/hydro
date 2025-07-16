# WebSocket Echo Server Example

This example demonstrates a simple WebSocket echo server built with DFIR. The server receives messages from clients 
and echoes them back, demonstrating basic WebSocket protocol handling and DFIR streaming patterns.

## Features

- WebSocket echo server that reflects all messages back to clients
- Handles all WebSocket message types (Text, Binary, Ping, Pong, Close)
- Responds to pings with pongs automatically
- DFIR streaming architecture for message processing
- Proper error handling and logging

## Running the Example

To run the echo server example, you'll need two terminals - one for the server and one for the client.

### Running the Server
In one terminal, start the echo server:
```shell
cargo run -p dfir_rs --example echo_server_websocket
```

The server will start listening on `127.0.0.1:8080` by default.

### Running the Client
You can test the WebSocket echo server using several methods:

#### Option 1: Using wscat (Recommended)
```shell
# Install wscat if you don't have it
npm install -g wscat

# Connect to the echo server
wscat -c ws://127.0.0.1:8080

# Type messages and see them echoed back
> Hello!
< Echo: Hello!
```

#### Option 2: Using the test client
The included test client sends a series of test messages automatically:
```shell
# Navigate to the client file and run it directly
cd dfir_rs/examples/echo_server_websocket
cargo run --bin client  # Note: This may require adding client as a binary target
```

#### Option 3: Using any WebSocket client
You can use any WebSocket client tool or library to connect to `ws://127.0.0.1:8080`.

## Message Types Handled

The echo server handles all WebSocket message types:

- **Text messages**: Echoed back with "Echo: " prefix
- **Binary messages**: Echoed back as-is
- **Ping messages**: Responded to with Pong messages
- **Pong messages**: Logged and acknowledged
- **Close messages**: Properly handled for connection termination

## Example Session

1. Start the server: `cargo run -p dfir_rs --example echo_server_websocket`
2. Connect with client: `cargo run -p dfir_rs --bin echo_server_websocket --bin client`
3. Watch as the client sends test messages and receives echoed responses
4. Or use `wscat` to interact manually

## Architecture

The example demonstrates DFIR's streaming operators:
- `source_stream()` to receive incoming WebSocket messages
- `map()` to transform messages (adding "Echo: " prefix)
- `filter_map()` for error handling
- `dest_sink()` to send responses back to clients
- Pattern matching on WebSocket message types
