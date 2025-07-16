use std::net::SocketAddr;

use clap::{Parser, ValueEnum};
use dfir_rs::lang::graph::{WriteConfig, WriteGraphType};
use dfir_rs::tokio;
use dfir_rs::util::ipv4_resolve;

mod client;
mod server;

/// HTTP server and client example using DFIR
#[dfir_rs::main]
async fn main() {
    // Parse command line arguments
    let opts = Opts::parse();

    // Run the server or the client based on the role provided in the command-line arguments.
    match opts.role {
        Role::Server => {
            server::run_server(&opts).await;
        }
        Role::Client => {
            client::run_client(&opts).await;
        }
    }
}

/// A simple HTTP server & client using DFIR.
#[derive(Parser, Debug)]
pub struct Opts {
    /// The role this application process should assume.
    #[clap(value_enum, long, default_value = "server")]
    pub role: Role,

    /// The server's network address. The server listens on this address. The client sends requests
    /// to this address. Format is `"ip:port"`.
    #[clap(long, value_parser = ipv4_resolve, default_value = DEFAULT_SERVER_ADDRESS)]
    pub address: SocketAddr,

    /// If specified, a graph representation of the flow used by the program will be
    /// printed to the console in the specified format.
    #[clap(long)]
    pub graph: Option<WriteGraphType>,

    #[clap(flatten)]
    pub write_config: Option<WriteConfig>,
}

/// The default server address & port on which the server listens for incoming HTTP requests.
pub const DEFAULT_SERVER_ADDRESS: &str = "localhost:3000";

/// A running application can assume one of these roles.
#[derive(Clone, ValueEnum, Debug)]
pub enum Role {
    Client,
    Server,
}
