use clap::Parser;
use ureq::{Agent, AgentBuilder};

use crate::cli_def::*;
use crate::http_ext::basic_auth;
use crate::kv_commons::*;
use crate::{http_ext::TokenAuthHeaderMiddleware, kv_commons::KVRemoteSource};

const AUTH_HEADER: &str = "Authorization";

#[derive(Parser, Debug)]
/// Subset of etcd specific commands
pub struct EtcdCommandConfig {
    /// Consul token for authentication
    #[arg(
        short = 'c',
        long = "creds",
        env = "ETCD_CREDENTIALS",
        help = "Etcd credentials for authentication. Value must be a base64 encoded 'user:password' string. Leave blank to skip authentication"
    )]
    pub token: Option<String>,

    /// Consul url
    #[arg(
        short = 'u',
        long = "url",
        env = "ETCD_ADDR",
        help = "Etcd remote address",
        default_value_t = String::from("http://127.0.0.1:2379")
    )]
    pub url: String,

    // Key separator add here
    // pub key_separator: u8
    /// Etcd command to execute
    #[command(subcommand)]
    pub kv_command: Option<KVSubs>,
}

pub struct EtcdRemote<'a> {
    pub config: &'a EtcdCommandConfig,
    pub agent: Agent,
}

impl<'a> EtcdRemote<'a> {
    /// Ctor for [`EtcdRemote`]
    pub fn new(config: &'a EtcdCommandConfig, agent_builder: AgentBuilder) -> Self {
        let authorizer = TokenAuthHeaderMiddleware::new(
            AUTH_HEADER.to_owned(),
            config.token.as_ref().map(basic_auth),
        );
        Self {
            config,
            agent: agent_builder.middleware(authorizer).build(),
        }
    }
}

impl<'a> KVRemoteSource for EtcdRemote<'a> {
    fn execute_kv_command(&self) {
        todo!()
    }

    fn list(&self, list_cfg: ListCmdConfig) -> Result<Vec<String>, KVError> {
        todo!()
    }

    fn read_path(&self, read_cfg: ReadCmdConfig) -> Result<KVValue, KVError> {
        todo!()
    }

    fn write_path(&self, write_cfg: WriteCmdConfig) -> Result<(), KVError> {
        todo!()
    }
}
