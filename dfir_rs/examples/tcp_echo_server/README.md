Simple TCP echo server example.

To run the example:
```
cargo run -p dfir_rs --example tcp_echo_server
```

The server will start listening on `localhost:3001`. You can test it using telnet or netcat:

```bash
telnet localhost 3001
# or
nc localhost 3001
```

Type messages and they will be echoed back with an "Echo: " prefix.

This example demonstrates:
- TCP connection handling with proper LocalSet usage
- Stream-based message processing with DFIR's declarative dataflow model
- Error handling using demux for success/error paths
- Connection state management for reliable TCP communication

Adding the `--graph <graph_type>` flag to the end of the command line above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).
