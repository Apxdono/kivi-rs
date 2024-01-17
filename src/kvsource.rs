use std::borrow::ToOwned;
use std::{error::Error, fmt::Display};

use crate::cli_def::{ListCmdConfig, ReadCmdConfig, WriteCmdConfig};
use edit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVValue {
    pub value: String,
    pub path: String,
}

impl KVValue {
    /// Send current value as buffer to system editor.
    ///
    /// Return new value after edit is complete.
    pub fn inline_edit_value(&self) -> Result<String, KVError> {
        return edit::edit(self.value.to_owned()).or_else(|err| KVError::wrap_as_write_err(err));
    }
}

/// Useful empty value
pub const EMPTY_KV_VALUE: KVValue = KVValue {
    value: "".to_owned(),
    path: "".to_owned(),
};

/// Configuration that dictates how to print [`KVValue`].
#[derive(Debug, Clone, Copy)]
pub struct KVDisplayConfig {
    pub as_b64_encoded: bool,
}

/// Common errors that happen when one works with KV Storages.
#[derive(Debug, Serialize, Deserialize)]
pub enum KVError {
    PermissionErr,
    RemoteErr,
    AuthenticationErr,
    NoValueErr,
    ValueFormatErr,
    ValueWriteErr(String),
}

impl KVError {
    pub fn wrap_as_write_err<T>(err: impl Error) -> Result<T, KVError> {
        return Err(KVError::ValueWriteErr(format!("{}", err)));
    }
}

impl Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KVError::PermissionErr => write!(f, "Error: not enough permissions"),
            KVError::RemoteErr => write!(f, "Error: remote returned error"),
            KVError::AuthenticationErr => write!(f, "Error: remote authentication required"),
            KVError::NoValueErr => write!(f, "<unknown_value>"),
            KVError::ValueFormatErr => write!(f, "<err_value>"),
            KVError::ValueWriteErr(msg) => write!(f, "<file_error:{}>", msg),
        }
    }
}

/// Abstract trait suitable _(hopefully)_ for any Key Value storage.
pub trait KVRemoteSource {
    fn execute_kv_command(&self);

    fn list(&self, list_cfg: ListCmdConfig) -> Result<Vec<String>, KVError>;

    fn read_path(&self, read_cfg: ReadCmdConfig) -> Result<KVValue, KVError>;

    fn write_path(&self, write_cfg: WriteCmdConfig) -> Result<(), KVError>;
}
