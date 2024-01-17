use core::result::Result;
use std::fs;

use clap::Parser;
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder, Error, Response};

use crate::http_ext::TokenAuthHeaderMiddleware;
use crate::{
    cli_def::{KVSubs, ListCmdConfig, ReadCmdConfig, WriteCmdConfig},
    kv_commons::{KVDisplayConfig, KVError, KVRemoteSource, KVValue},
    utils::*,
};

const KV_API_PATH: &str = "/v1/kv/";
const FIRST_LEVEL_KEYS_PARAMS: &str = "?keys=true&separator=/";

/// Represents Consul KV source
pub struct ConsulRemote<'a> {
    pub config: &'a ConsulCommandConfig,
    pub agent: Agent,
}

impl<'a> ConsulRemote<'a> {
    /// Ctor for [`ConsulRemote`]
    pub fn new(config: &'a ConsulCommandConfig, agent_builder: AgentBuilder) -> Self {
        let authorizer =
            TokenAuthHeaderMiddleware::new("X-CONSUL-TOKEN".to_owned(), config.token.to_owned());
        Self {
            config,
            agent: agent_builder.middleware(authorizer).build(),
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
    value: Option<String>,
    create_index: u32,
    modify_index: u32,
}

/// Subset of Consul specific commands
#[derive(Parser, Debug)]
#[command(
    about = "Connect to Consul Server",
    long_about = "Connect to Consul Server"
)]
pub struct ConsulCommandConfig {
    /// Consul token for authentication
    #[arg(
        short = 't',
        long = "token",
        env = "CONSUL_HTTP_TOKEN",
        help = "Consul token to supply, leave blank to skip authentication"
    )]
    pub token: Option<String>,

    /// Consul url
    #[arg(
        short = 'u',
        long = "url",
        env = "CONSUL_HTTP_ADDR",
        help = "Consul token to supply, leave blank to skip authentication",
        default_value_t = String::from("http://127.0.0.1:8500")
    )]
    pub url: String,

    /// Consul command to execute
    #[command(subcommand)]
    pub kv_command: KVSubs,
}

/// Converts internal [`ConsulValue`] to [`KVValue`].
///
/// * `display_cfg` - [`KVDisplayConfig`] that determines how to represent the display value (b64 or plain string).  
fn to_kv_value(display_cfg: KVDisplayConfig) -> impl Fn(ConsulValue) -> KVValue {
    return move |consul_val: ConsulValue| {
        let extractor = match display_cfg.as_b64_encoded {
            true => identity_str,
            false => decodeb64_safe,
        };

        let extracted = match consul_val.value {
            None => "".to_owned(),
            Some(st) => extractor(&st),
        };

        return KVValue {
            path: consul_val.key.to_string(),
            value: extracted,
        };
    };
}

/// Linter of Consul keys. Used to remove the prefix after list command is complete.
///
/// This is done to align response from Consul with other KV storages that return only immediate
/// child node names instead of full paths.
///
/// See [`create_str_linter()`]
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

// Consul Response is always a JSON array of items
fn process_consul_response(
    response: Response,
    display_cfg: KVDisplayConfig,
) -> Result<KVValue, KVError> {
    let kv_value_mapper = |vec_consul_vals: Vec<ConsulValue>| -> Vec<KVValue> {
        return vec_consul_vals
            .into_iter()
            .map(to_kv_value(display_cfg))
            .collect::<Vec<KVValue>>();
    };

    let result_items = response
        .into_json::<Vec<ConsulValue>>()
        .map(kv_value_mapper);

    return match result_items {
        Err(_) => Err(KVError::ValueFormatErr),
        Ok(items) => match items.first() {
            Some(item) => Ok(item.clone()),
            None => Err(KVError::NoValueErr),
        },
    };
}

impl<'a> ConsulRemote<'a> {
    /**
    Create properly formed Consul KV HTTP API URL.

    Suffix is a relative path string that captures all path chunks that follow /v1/kv/ base path.
    Leading `/` in suffix is removed before url is formed.

    See [build_url()]

    Examples:

    ```
    assert_eq!("http://127.0.0.1:8500/v1/kv/some/value/under/path", self.to_consul_url("some/value/under/path");

    assert_eq!("http://127.0.0.1:8500/v1/kv/other/value/under/path", self.to_consul_url("/other/value/under/path");
    ````
     */
    fn to_consul_url(&self, suffix: &String) -> String {
        return build_url(&self.config.url, KV_API_PATH, suffix);
    }

    fn write_to_path(&self, write_cfg: WriteCmdConfig, content: String) -> Result<(), KVError> {
        let consul_url = self.to_consul_url(&write_cfg.path);
        let res_response = self.agent.put(&consul_url).send_bytes(content.as_bytes());

        return match res_response {
            Err(status) => remap_consul_errors(status),
            Ok(_) => Ok(()),
        };
    }
}

impl<'a> KVRemoteSource for ConsulRemote<'a> {
    fn execute_kv_command(&self) {
        match &self.config.kv_command {
            KVSubs::Read(read_cmd) => {
                let read_res = self.read_path(read_cmd.clone());
                match read_res {
                    Ok(kv_val) => print!("{}", kv_val),
                    Err(err) => eprintln!("{err}"),
                }
            }
            KVSubs::List(list_cmd) => {
                let list_res = self.list(list_cmd.clone());
                match list_res {
                    Ok(keys) => println!("{}", keys.join("\n")),
                    Err(err) => eprintln!("{err}"),
                }
            }
            KVSubs::Write(write_cmd) => {
                let write_res = self.write_path(write_cmd.clone());
                if let Err(err) = write_res {
                    eprintln!("{err}");
                }
            }
        }
    }

    fn list(&self, list_cfg: ListCmdConfig) -> Result<Vec<String>, KVError> {
        let consul_url = self.to_consul_url(&list_cfg.prefix) + FIRST_LEVEL_KEYS_PARAMS;
        let request = self.agent.get(&consul_url);

        let res_response = request.call();

        return match res_response {
            Err(status) => remap_consul_errors(status),
            Ok(response) => Ok(response
                .into_json::<Vec<String>>()
                .map(create_prefix_iter_linter(list_cfg.prefix))
                .unwrap()),
        };
    }

    fn read_path(&self, read_cfg: ReadCmdConfig) -> Result<KVValue, KVError> {
        let consul_url = self.to_consul_url(&read_cfg.path);
        let request = self.agent.get(&consul_url);

        let res_response = request.call();
        let kv_display_config = KVDisplayConfig {
            as_b64_encoded: read_cfg.is_encoded,
        };

        return match res_response {
            Err(status) => remap_consul_errors(status),
            Ok(response) => process_consul_response(response, kv_display_config),
        };
    }

    fn write_path(&self, write_cfg: WriteCmdConfig) -> Result<(), KVError> {
        let write_new_value = |content| self.write_to_path(write_cfg.clone(), content);

        return if write_cfg.is_in_place_edit {
            self.read_path(ReadCmdConfig {
                is_encoded: false,
                path: write_cfg.path.to_owned(),
            })
            .and_then(|kv_val| kv_val.inline_edit_value())
            .and_then(write_new_value)
        } else {
            match write_cfg.data_file.to_owned() {
                Some(file) => {
                    return fs::read_to_string(file)
                        .or_else(KVError::wrap_as_write_err)
                        .and_then(write_new_value);
                }
                None => Ok(()),
            }
        };
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
