use clap::{Parser, Subcommand};

use crate::consul_remote::ConsulCommandConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = false)]
pub struct Cli {
    #[arg(short = 'l', long = "log", default_value_t = String::from("info"))]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Option<Subs>,
}

#[derive(Subcommand)]
pub enum Subs {
    Consul(ConsulCommandConfig),
}

#[derive(Subcommand, Debug)]
pub enum KVSubs {
    Read(ReadCmdConfig),
    Write(WriteCmdConfig),
    List(ListCmdConfig),
}

#[derive(Parser, Clone, Debug)]
#[command(about = "Read value under path", long_about = "Read value under path")]
pub struct ReadCmdConfig {
    #[arg(
        short = 'e',
        long = "encoded",
        help = "encode value as base64 string",
        action
    )]
    pub is_encoded: bool,

    #[arg(help = "Value path")]
    pub path: String,
}

#[derive(Parser, Clone, Debug)]
#[command(
    about = "Write value under path",
    long_about = "Write value under path"
)]
pub struct WriteCmdConfig {
    #[arg(
        short = 'i',
        long = "inplace",
        help = "read and modify existing remote value. Uses system $EDITOR for editing",
        default_value_t = false,
        action
    )]
    pub is_in_place_edit: bool,

    #[arg(
        short = 'd',
        long = "data",
        help = "File content to write. Ignored if 'inplace' write"
    )]
    pub data_file: Option<String>,

    #[arg(help = "Value path")]
    pub path: String,
}

#[derive(Parser, Clone, Debug)]
#[command(
    about = "List all prefix subfolders",
    long_about = "List all prefix subfolders"
)]
pub struct ListCmdConfig {
    #[arg(help = "Target prefix")]
    pub prefix: String,
}
