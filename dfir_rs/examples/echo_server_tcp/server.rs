//[imports]//
use std::net::SocketAddr;

use chrono::prelude::*;
use dfir_rs::dfir_syntax;
use dfir_rs::scheduled::graph::Dfir;
use dfir_rs::util::bind_tcp_lines;
use tokio::task::LocalSet;

use crate::Opts;
use crate::helpers::print_graph;
use crate::protocol::EchoMsg;
//[/imports]//

/// Runs the server. The server is a long-running process that listens for messages and echoes
/// them back the client.
pub(crate) async fn run_server(opts: Opts) {
    // If a server address & port are provided as command-line inputs, use those, else use the
    // default.
    let server_address = opts.address;

    println!("Starting server on {:?}", server_address);

    //[run_server]//
    // TCP operations require LocalSet for proper execution.
    // We need to bind and run the flow within the same LocalSet scope.
    LocalSet::new()
        .run_until(async {
            //[bind_tcp]//
            // Bind a server-side socket to requested address and port. If "0" was provided as the port, the
            // OS will allocate a port and the actual port used will be available in `actual_server_addr`.
            //
            // `outbound` is a `TcpSink`, we use it to send messages. `inbound` is `TcpStream`, we use it
            // to receive messages.
            let (outbound, inbound, actual_server_addr) = bind_tcp_lines(server_address).await;
            //[/bind_tcp]//

            println!("Server is live! Listening on {:?}", actual_server_addr);

            //[dfir_flow]//
            // The skeletal DFIR spec for a TCP server.
            let mut flow: Dfir = dfir_syntax! {

                //[inbound_processing]//
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
                    -> map(Result::unwrap); // If the deserialization was unsuccessful, this line will panic.
                //[/inbound_processing]//

                //[outbound_setup]//
                // For TCP lines, we need to serialize manually and convert to String
                // since the lines codec expects String input, not Bytes
                outbound_chan = dest_sink(outbound);
                //[/outbound_setup]//

                //[echo_logic]//
                // Handle the inbound messages.
                inbound_chan
                    -> inspect(|(m, a): &(EchoMsg, SocketAddr)| println!("{}: Got {:?} from {:?}", Utc::now(), m, a)) // For debugging purposes.
                    -> map(|(EchoMsg { payload, ts: _ }, sender_addr)| {
                        let echo_msg = EchoMsg { payload, ts: Utc::now() };
                        let serialized = serde_json::to_string(&echo_msg).unwrap();
                        (serialized, sender_addr)
                    })
                    -> [0]outbound_chan;
                //[/echo_logic]//
            };
            //[/dfir_flow]//

            // If a graph was requested to be printed, print it.
            if let Some(graph) = opts.graph {
                print_graph(&flow, graph, opts.write_config);
            }

            // Run the server.
            flow.run_async().await;
        })
        .await;
    //[/run_server]//
}
