mod cli_def;
mod consul_remote;
mod etcd_remote;
mod http_ext;
mod kv_commons;
mod utils;

use base64::write::StrConsumer;
use clap::Parser;
use etcd_remote::EtcdRemote;
use std::time::Duration;
use ureq::AgentBuilder;

use crate::{cli_def::Cli, cli_def::Subs};
use crate::{consul_remote::ConsulRemote, kv_commons::KVRemoteSource};

const DEFAULT_KO_TIME: Duration = Duration::from_secs(5);

fn build_client() -> AgentBuilder {
    return AgentBuilder::new()
        .timeout_connect(DEFAULT_KO_TIME)
        .timeout_read(DEFAULT_KO_TIME);
}

fn main() {
    let cli = Cli::parse();
    let client_builder: AgentBuilder = build_client();
    match &cli.command {
        Some(Subs::Consul(cfg)) => {
            let consul = ConsulRemote::new(cfg, client_builder);
            consul.execute_kv_command();
        }
        Some(Subs::Etcd(cfg)) => {
            let etcd = EtcdRemote::new(cfg, client_builder);
            etcd.execute_kv_command();
        }
        None => println!("Nothing happened"),
    }
}
