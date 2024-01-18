use clap::{Parser, Subcommand};

use crate::consul_remote::ConsulCommandConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = false)]
pub struct Cli {
    #[arg(short = 'l', long = "log", default_value_t = String::from("info"))]
    /// Set application log level
    pub log_level: String,

    #[command(subcommand)]
    pub command: Option<Subs>,
}

#[derive(Subcommand, Debug)]
#[command(subcommand_required = true)]
pub enum Subs {
    Consul(ConsulCommandConfig),
}

#[derive(Subcommand, Debug)]
#[command(subcommand_required = true)]
pub enum KVSubs {
    Read(ReadCmdConfig),
    Write(WriteCmdConfig),
    List(ListCmdConfig),
}

#[derive(Parser, Clone, Debug)]
/// Read value under storage path
pub struct ReadCmdConfig {
    #[arg(short = 'e', long = "encoded", action)]
    /// encode value as base64 string
    pub is_encoded: bool,

    #[arg()]
    /// value path
    pub path: String,
}

#[derive(Parser, Clone, Debug)]
/// Write value under storage path
pub struct WriteCmdConfig {
    #[arg(short = 'i', long = "inline", default_value_t = false, action)]
    /// read and modify existing remote value. Uses system $EDITOR for editing
    pub is_inline_edit: bool,

    #[arg(short = 'd', long = "data")]
    /// File content to write. Ignored if 'inline' write
    pub data_file: Option<String>,

    #[arg()]
    /// value path
    pub path: String,
}

#[derive(Parser, Clone, Debug)]
/// List all prefix child nodes
pub struct ListCmdConfig {
    #[arg()]
    /// target prefix
    pub prefix: String,
}
