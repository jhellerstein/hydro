use std::net::SocketAddr;

use clap::{Parser, ValueEnum};
use client::run_client;
use dfir_rs::lang::graph::{WriteConfig, WriteGraphType};
use dfir_rs::util::ipv4_resolve;
use server::run_server;

mod client;
mod server;

#[derive(Clone, Copy, ValueEnum, Debug, Eq, PartialEq)]
pub enum Role {
    Client,
    Server,
}

pub fn default_server_address() -> SocketAddr {
    ipv4_resolve("localhost:8080").unwrap()
}

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(long)]
    pub name: String,
    #[clap(value_enum, long)]
    pub role: Role,
    #[clap(long, value_parser = ipv4_resolve)]
    pub address: Option<SocketAddr>,
    #[clap(long)]
    pub graph: Option<WriteGraphType>,
    #[clap(flatten)]
    pub write_config: Option<WriteConfig>,
}

#[dfir_rs::main]
async fn main() {
    let opts = Opts::parse();

    match opts.role {
        Role::Client => {
            run_client(opts).await;
        }
        Role::Server => {
            run_server(opts).await;
        }
    }
}
