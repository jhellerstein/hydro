Simple TCP echo server and client example.

To run the example, open 2 terminals.

In one terminal run the server like so:
```
cargo run -p dfir_rs --example echo_server_tcp -- --role server --address localhost:3001
```

In another terminal run a client:
```
cargo run -p dfir_rs --example echo_server_tcp -- --role client --address localhost:3001
```

If you type in the client terminal the message will be sent to the server, echo'd back to the client and printed with a checksum and server timestamp.

This example demonstrates:
- TCP connection handling with proper LocalSet usage
- Stream-based message processing with DFIR's declarative dataflow model
- Error handling using demux for success/error paths
- Connection state management for reliable TCP communication

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).
