use std::net::SocketAddr;

use chrono::prelude::*;
use dfir_rs::dfir_syntax;
use dfir_rs::util::connect_tcp_lines;
use tokio::task::LocalSet;

use crate::Opts;
use crate::helpers::print_graph;
use crate::protocol::EchoMsg;

/// Runs the client. The client is a long-running process that reads stdin, and sends messages that
/// it receives to the server. The client also prints any messages it receives to stdout.
pub(crate) async fn run_client(opts: Opts) {
    // Use the server address that was provided in the command-line arguments, or use the default
    // if one was not provided.
    let server_addr = opts.address;
    assert_ne!(
        0,
        server_addr.port(),
        "Client cannot connect to server port 0."
    );

    println!("Client is live! Connecting to server {:?}", server_addr);

    // TCP operations require LocalSet for proper execution.
    LocalSet::new().run_until(async {
        // Connect to the TCP server. The TCP connect functions return
        // sinks/streams that accept (data, address) pairs.
        let (outbound, inbound) = connect_tcp_lines();

        // The skeletal DFIR spec for a TCP client.
        let mut flow = dfir_syntax! {

            // Whenever a serialized message is received by the application from a particular address,
            // a (serialized_payload, address_of_sender) pair is emitted by the `inbound` stream.
            //
            // For TCP lines, we manually deserialize the JSON string to EchoMsg
            inbound_chan = source_stream(inbound)
                -> map(|result: Result<(String, SocketAddr), _>| {
                    result.map(|(line, addr)| {
                        let echo_msg: EchoMsg = serde_json::from_str(&line).unwrap();
                        (echo_msg, addr)
                    })
                })
                -> map(Result::unwrap);  // If the deserialization was unsuccessful, this line will panic.

            // For TCP lines, we need to serialize manually and convert to String
            // since the lines codec expects String input, not Bytes
            outbound_chan = dest_sink(outbound);

            // Print all messages for debugging purposes.
            inbound_chan
                -> for_each(|(m, a): (EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), m, a));

            // Consume input from stdin and send to server as an `EchoMsg`.
            source_stdin() // A stream of lines from stdin.
                -> map(|l| {
                    let echo_msg = EchoMsg { payload: l.unwrap(), ts: Utc::now() };
                    let serialized = serde_json::to_string(&echo_msg).unwrap();
                    (serialized, server_addr)
                })
                -> outbound_chan; // Send it to the server
        };

        // If a graph was requested to be printed, print it.
        if let Some(graph) = opts.graph {
            print_graph(&flow, graph, opts.write_config);
        }

        // Run the client. This is an async function, so we need to await it.
        flow.run_async().await
    }).await;
}
