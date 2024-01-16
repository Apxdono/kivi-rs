use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::cli_def::{ListCmdConfig, ReadCmdConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVValue {
    pub value: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub struct KVDisplayConfig {
    pub as_b64_encoded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KVError {
    PermissionErr,
    RemoteErr,
    AuthenticationErr,
    NoValueErr,
    ValueFormatErr,
}

impl Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            KVError::PermissionErr => write!(f, "Error: not enough permissions"),
            KVError::RemoteErr => write!(f, "Error: remote returned error"),
            KVError::AuthenticationErr => write!(f, "Error: remote authentication required"),
            KVError::NoValueErr => write!(f, "<unknown_value>"),
            KVError::ValueFormatErr => write!(f, "<err_value>"),
        }
    }
}
pub trait KVSource {
    fn execute_kv_command(&self);

    fn list(&self, list_cfg: ListCmdConfig) -> Result<Vec<String>, KVError>;

    fn read_path(&self, read_cfg: ReadCmdConfig) -> Result<KVValue, KVError>;

    fn write_path(&self, path: String, value: KVValue) -> Result<(), KVError>;
}
