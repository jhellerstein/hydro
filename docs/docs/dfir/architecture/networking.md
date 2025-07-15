---
sidebar_position: 3
---

# Networking with DFIR

DFIR provides comprehensive networking support for building distributed systems that process data streams over UDP, TCP, and HTTP protocols. This guide covers the fundamental networking patterns and demonstrates how to build servers and clients using DFIR's declarative dataflow programming model.

## Overview

DFIR's networking model follows these key principles:

- **Stream-oriented**: Network data flows as streams through DFIR operators
- **Declarative**: Network logic is expressed as dataflow graphs
- **Protocol-agnostic**: Similar patterns work across UDP, TCP, and HTTP
- **Error-aware**: Built-in error handling with demux routing
- **Address-aware**: All network operations include source/destination addressing

## Core Networking Patterns

### Basic Server Pattern

Nearly all DFIR network servers follow this pattern:

```
bind_protocol();
source_stream() -> demux() -> processing -> dest_sink()
```

### Basic Client Pattern

Most DFIR network clients follow this pattern:

```
connect_protocol();
source_stream() -> demux() -> processing
```

## UDP Networking

UDP provides connectionless, best-effort message delivery. DFIR's UDP support is ideal for high-throughput, low-latency applications.

### UDP Echo Server

```rust
!!raw-loader!../../../dfir_rs/examples/echo_server/server.rs?getLines(code, 'imports')
```

The UDP echo server demonstrates the fundamental DFIR networking pattern:

```rust
!!raw-loader!../../../dfir_rs/examples/echo_server/server.rs?getLines(code, 'dfir_flow')
```

Key UDP characteristics:
- **Connectionless**: Each message is independent
- **Best-effort**: No delivery guarantees
- **Low overhead**: Minimal protocol overhead
- **High throughput**: Suitable for streaming data

## TCP Networking

TCP provides reliable, connection-oriented byte streams. DFIR's TCP support handles connection management automatically while exposing a stream-based interface.

### TCP Echo Server

```rust
!!raw-loader!../../../dfir_rs/examples/tcp_echo_server.rs?getLines(code, 'imports')
```

#### LocalSet Requirement for TCP Operations

TCP operations in DFIR require the use of tokio's `LocalSet` for proper execution. `LocalSet` is a special execution context in tokio that allows tasks to run on a single thread, which is necessary for DFIR's internal connection management.

**What is LocalSet?**
- LocalSet is a tokio utility that provides a single-threaded execution context for async tasks
- Unlike regular tokio tasks that can run on any thread in the runtime's thread pool, LocalSet tasks are guaranteed to run on the same thread
- This is required because DFIR's TCP implementation uses thread-local storage and spawns local tasks that cannot be moved between threads

**Why TCP needs LocalSet:**
- DFIR's `bind_tcp_bytes`, `bind_tcp_lines`, and related TCP functions use `spawn_local` internally
- `spawn_local` requires being called from within a LocalSet context
- This design choice ensures proper connection state management and thread safety

Here's how to use TCP with LocalSet:

```rust
!!raw-loader!../../../dfir_rs/examples/tcp_echo_server.rs?getLines(code, 'bind_tcp')
```

The TCP server processes connection streams through DFIR:

```rust
!!raw-loader!../../../dfir_rs/examples/tcp_echo_server.rs?getLines(code, 'dfir_flow')
```

Key TCP characteristics:
- **Connection-oriented**: Explicit connection establishment
- **Reliable**: Guaranteed in-order delivery
- **Flow control**: Automatic congestion management
- **Stateful**: Connection state maintained automatically

## HTTP Networking

HTTP builds on TCP to provide request-response semantics with rich metadata. DFIR's HTTP support includes request/response parsing and routing.

**Note:** Like TCP operations, HTTP operations in DFIR also require `LocalSet` since they use the same underlying TCP infrastructure. All HTTP servers and clients must be run within a `LocalSet` context.

### HTTP Server

```rust
!!raw-loader!../../../dfir_rs/examples/http_server.rs?getLines(code, 'imports')
```

HTTP servers bind to addresses and receive structured requests. Note the LocalSet wrapper required for HTTP operations:

```rust
!!raw-loader!../../../dfir_rs/examples/http_server.rs?getLines(code, 'bind_server')
```

HTTP request routing uses demux for efficient path-based dispatch:

```rust
!!raw-loader!../../../dfir_rs/examples/http_server.rs?getLines(code, 'route_handlers')
```

Response generation follows the same patterns:

```rust
!!raw-loader!../../../dfir_rs/examples/http_server.rs?getLines(code, 'response_union')
```

### HTTP Client

```rust
!!raw-loader!../../../dfir_rs/examples/http_client.rs?getLines(code, 'imports')
```

HTTP clients connect and send structured requests:

```rust
!!raw-loader!../../../dfir_rs/examples/http_client.rs?getLines(code, 'connect_client')
```

Client request processing:

```rust
!!raw-loader!../../../dfir_rs/examples/http_client.rs?getLines(code, 'send_requests')
```

Response handling with error processing:

```rust
!!raw-loader!../../../dfir_rs/examples/http_client.rs?getLines(code, 'response_processing')
```

Key HTTP characteristics:
- **Request-response**: Structured message exchange
- **Stateless**: Each request is independent
- **Rich metadata**: Headers, methods, status codes
- **Content-aware**: Body parsing and generation

## Error Handling

All DFIR networking uses consistent error handling patterns with demux:

```rust
// UDP error handling
messages = source_stream(socket_recv)
    -> demux(|result, var_args!(success, error)| {
        match result {
            Ok((data, addr)) => success.give((data, addr)),
            Err(e) => error.give(e),
        }
    });

messages[success] -> /* process valid messages */;
messages[error] -> for_each(|e| eprintln!("Error: {}", e));
```

## Address Management

DFIR networking operators always include address information:

- **UDP**: `(data, SocketAddr)` tuples for each packet
- **TCP**: `(data, SocketAddr)` tuples identifying the connection
- **HTTP**: `(HttpRequest/Response, SocketAddr)` tuples with client info

## Performance Considerations

### UDP Performance
- Use `bind_udp_bytes()` for maximum throughput
- Consider message size vs. fragmentation
- Implement application-level reliability if needed

### TCP Performance
- Connection reuse is handled automatically
- Use appropriate buffer sizes for your workload
- Consider connection pooling for high-concurrency clients

### HTTP Performance
- Leverage HTTP keep-alive for connection reuse
- Use appropriate content encoding
- Consider HTTP/2 for multiplexing (future enhancement)

## Advanced Patterns

### Protocol Composition

DFIR's uniform stream model enables protocol composition:

```rust
// HTTP over UDP tunnel
udp_stream -> map(parse_http) -> demux(route_requests) -> map(generate_response) -> udp_sink
```

### Multi-protocol Servers

Single DFIR graphs can handle multiple protocols:

```rust
dfir_syntax! {
    // Combine UDP and TCP streams
    udp_messages = source_stream(udp_recv) -> map(tag_udp);
    tcp_messages = source_stream(tcp_recv) -> map(tag_tcp);
    
    all_messages = union(udp_messages, tcp_messages) -> process_unified;
}
```

### Load Balancing

DFIR's demux operator enables sophisticated routing:

```rust
// Route by client address
requests -> demux(|(req, addr), var_args!(server1, server2, server3)| {
    match addr.ip().octets()[3] % 3 {
        0 => server1.give((req, addr)),
        1 => server2.give((req, addr)),
        _ => server3.give((req, addr)),
    }
});
```

## Best Practices

1. **Always handle errors**: Use demux to separate success/error paths
2. **Include addresses**: Maintain client addressing throughout processing
3. **Use appropriate protocols**: UDP for throughput, TCP for reliability, HTTP for structure
4. **Consider backpressure**: DFIR handles flow control automatically
5. **Test incrementally**: Start with simple echo servers and add complexity
6. **Monitor performance**: Use DFIR's built-in observability features

## LocalSet Requirements Summary

Understanding when LocalSet is required for DFIR networking:

| Protocol | LocalSet Required | Reason |
|----------|------------------|---------|
| **UDP** | ❌ No | Uses regular tokio tasks and thread-safe operations |
| **TCP** | ✅ Yes | Internal `spawn_local` usage for connection management |
| **HTTP** | ✅ Yes | Built on TCP infrastructure, inherits LocalSet requirement |

**When to use LocalSet:**
```rust
use tokio::task::LocalSet;

#[tokio::main]
async fn main() {
    let local = LocalSet::new();
    
    // TCP and HTTP operations must be spawned within LocalSet
    local.spawn_local(async {
        // Your DFIR TCP/HTTP code here
        let (sink, stream, addr) = dfir_rs::util::bind_tcp_lines(addr).await;
        // ... rest of your DFIR graph
    });
    
    // Run the LocalSet
    local.await;
}
```

**Why this matters:**
- Attempting to use TCP/HTTP operations outside LocalSet will panic at runtime
- UDP operations work fine in regular tokio contexts
- LocalSet ensures proper thread-local task execution required by DFIR's TCP implementation

## Examples

For complete working examples, see:
- [UDP Echo Server](https://github.com/hydro-project/hydro/blob/main/dfir_rs/examples/echo_server/server.rs)
- [TCP Echo Server](https://github.com/hydro-project/hydro/blob/main/dfir_rs/examples/tcp_echo_server.rs)
- [HTTP Server](https://github.com/hydro-project/hydro/blob/main/dfir_rs/examples/http_server.rs)
- [HTTP Client](https://github.com/hydro-project/hydro/blob/main/dfir_rs/examples/http_client.rs)

This networking foundation enables building sophisticated distributed systems with DFIR's declarative, stream-oriented programming model.