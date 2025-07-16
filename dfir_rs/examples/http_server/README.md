Simple HTTP server and client example.

To run the example, open 2 terminals.

In one terminal run the server like so:
```
cargo run -p dfir_rs --example http_server
```

In another terminal run a client:
```
cargo run -p dfir_rs --example http_server -- --role client
```

The server will start listening on `http://localhost:3000` with several endpoints:
- `GET /` - Returns a welcome page
- `GET /api/hello` - Returns a JSON greeting
- `POST /api/echo` - Echoes back the JSON request body
- Any other path returns a 404

You can also test the server with curl:
```bash
curl http://localhost:3000/
curl http://localhost:3000/api/hello
curl -X POST -H "Content-Type: application/json" -d '{"test": "data"}' http://localhost:3000/api/echo
```

Both the server and client demonstrate HTTP request/response handling using DFIR's declarative dataflow programming model with proper LocalSet usage for TCP-based operations.

Adding the `--graph <graph_type>` flag to the end of the command lines above will print out a node-and-edge diagram of the program. Supported values for `<graph_type>` include [mermaid](https://mermaid-js.github.io/) and [dot](https://graphviz.org/doc/info/lang.html).
