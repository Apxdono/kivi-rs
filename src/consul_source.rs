use crate::{
    cli_def::{KVSubs, ListCmdConfig, ReadCmdConfig},
    kvsource::{KVDisplayConfig, KVError, KVSource, KVValue},
    utils::{build_url, create_str_linter, decodeb64_safe, identity_str},
};
use clap::Parser;
use core::result::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use ureq::{Agent, AgentBuilder, Error, Middleware, Request, Response};

const KV_API_PATH: &str = "/v1/kv/";
const FIRST_LEVEL_KEYS_PARAMS: &str = "?keys=true&separator=/";

/// Represents Consul KV source
pub struct ConsulSource<'a> {
    pub config: &'a ConsulCommandConfig,
    pub agent: Agent,
}

///
struct ConsulAuthMiddleware {
    token: Option<String>,
}

impl ConsulAuthMiddleware {
    fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

impl Middleware for ConsulAuthMiddleware {
    fn handle(&self, request: Request, next: ureq::MiddlewareNext) -> Result<Response, Error> {
        let req: Request = match &self.token {
            Some(token) => request.set("X-CONSUL-TOKEN", token.as_str()),
            _ => request,
        };
        next.handle(req)
    }
}

impl<'a> ConsulSource<'a> {
    pub fn new(config: &'a ConsulCommandConfig, agent_builder: AgentBuilder) -> Self {
        Self {
            config,
            agent: agent_builder
                .middleware(ConsulAuthMiddleware::new(config.token.to_owned()))
                .build(),
        }
    }
}

/// Represents stored/read Consul Value
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ConsulValue {
    lock_index: u8,
    key: String,
    flags: u8,
    value: String,
    create_index: u32,
    modify_index: u32,
}

#[derive(Parser, Debug)]
#[command(
    about = "Connect to Consul Server",
    long_about = "Connect to Consul Server"
)]
pub struct ConsulCommandConfig {
    #[arg(
        short = 't',
        long,
        env = "CONSUL_HTTP_TOKEN",
        help = "Consul token to supply, leave blank to skip authentication"
    )]
    pub token: Option<String>,
    #[arg(
        short='u',
        long,
        env = "CONSUL_HTTP_ADDR",
        default_value_t = String::from("http://127.0.0.1:8500"),
        help = "Consul address"
    )]
    pub url: String,

    #[command(subcommand)]
    pub kv_command: KVSubs,
}

fn to_kv_value(display_cfg: KVDisplayConfig) -> impl Fn(ConsulValue) -> KVValue {
    return move |consul_val: ConsulValue| {
        let extractor = match display_cfg.as_b64_encoded {
            true => identity_str,
            false => decodeb64_safe,
        };

        return KVValue {
            path: consul_val.key.to_string(),
            value: extractor(&consul_val.value),
        };
    };
}

fn create_prefix_iter_linter(prefix: String) -> impl Fn(Vec<String>) -> Vec<String> {
    let keys_linter = create_str_linter(Some(prefix.to_owned()), None, false);
    return move |keys: Vec<String>| -> Vec<String> {
        return keys
            .into_iter()
            .map(&keys_linter)
            .filter(|s| !s.is_empty())
            .collect();
    };
}

// Json is always an array of items
fn process_consul_response(
    response: Response,
    display_cfg: KVDisplayConfig,
) -> Result<KVValue, KVError> {
    let result_items =
        response
            .into_json::<Vec<ConsulValue>>()
            .map(|vec_consul_vals| -> Vec<KVValue> {
                vec_consul_vals
                    .into_iter()
                    .map(to_kv_value(display_cfg.clone()))
                    .collect()
            });

    return match result_items {
        Err(_) => Err(KVError::ValueFormatErr),
        Ok(items) => match items.first() {
            Some(&ref item) => Ok(item.clone()),
            None => Err(KVError::NoValueErr),
        },
    };
}

fn to_consul_url(consul: &ConsulSource, suffix: &String) -> String {
    return build_url(&consul.config.url, KV_API_PATH, suffix);
}

impl<'a> KVSource for ConsulSource<'a> {
    fn list(&self, list_cfg: ListCmdConfig) -> Result<Vec<String>, KVError> {
        let consul_url = to_consul_url(self, &list_cfg.prefix) + FIRST_LEVEL_KEYS_PARAMS;
        let request = self.agent.get(&consul_url);

        let res_response: Result<ureq::Response, ureq::Error> = request.call();

        return match res_response {
            Err(status) => remap_consul_errors(status),
            Ok(response) => Ok(response
                .into_json::<Vec<String>>()
                .map(create_prefix_iter_linter(list_cfg.prefix))
                .unwrap()),
        };
    }

    fn read_path(&self, read_cfg: ReadCmdConfig) -> Result<KVValue, KVError> {
        let consul_url = to_consul_url(self, &read_cfg.path);
        let request = self.agent.get(&consul_url);

        let res_response: Result<ureq::Response, ureq::Error> = request.call();
        let kv_display_config = KVDisplayConfig {
            as_b64_encoded: read_cfg.is_encoded,
        };

        return match res_response {
            Err(status) => remap_consul_errors(status),
            Ok(response) => process_consul_response(response, kv_display_config),
        };
    }

    fn write_path(&self, _path: String, _value: KVValue) -> Result<(), KVError> {
        todo!()
    }

    fn execute_kv_command(&self) {
        match &self.config.kv_command {
            KVSubs::Read(read_cmd) => {
                let read_res = self.read_path(read_cmd.clone());
                match read_res {
                    Ok(kv_val) => println!("{}", kv_val.value),
                    Err(err) => eprintln!("{err}"),
                }
            }
            KVSubs::List(list_cmd) => {
                let list_res = self.list(list_cmd.clone());
                match list_res {
                    Ok(keys) => {
                        let mut lock = io::stdout().lock();
                        keys.iter().for_each(|k| writeln!(lock, "{}", k).unwrap());
                    }
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    }
}

fn remap_consul_errors<T>(status: Error) -> Result<T, KVError> {
    match status {
        Error::Status(403, _) => Err(KVError::PermissionErr),
        Error::Status(401, _) => Err(KVError::AuthenticationErr),
        Error::Status(404, _) => Err(KVError::NoValueErr),
        Error::Transport(_) => Err(KVError::RemoteErr),
        Error::Status(_, _) => Err(KVError::RemoteErr),
    }
}
