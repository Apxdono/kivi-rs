use clap::Parser;
use std::time::Duration;
use ureq::AgentBuilder;

mod cli_def;
mod consul_source;
mod kvsource;
mod utils;

use cli_def::Subs;

use crate::{consul_source::ConsulSource, kvsource::KVSource};

const DEFAULT_KO_TIME: Duration = Duration::from_secs(5);

fn build_client() -> AgentBuilder {
    return AgentBuilder::new()
        .timeout_connect(DEFAULT_KO_TIME)
        .timeout_read(DEFAULT_KO_TIME);
}

fn main() {
    let cli = cli_def::Cli::parse();
    let client_builder: AgentBuilder = build_client();
    match &cli.command {
        Some(Subs::Consul(cfg)) => {
            let consul = ConsulSource::new(cfg, client_builder);
            consul.execute_kv_command();
        }
        None => println!("Nothing happened"),
    }
}
