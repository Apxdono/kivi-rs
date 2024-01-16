use clap::{Parser, Subcommand};

use crate::consul_source::ConsulCommandConfig;

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
    about = "List all prefix subfolders",
    long_about = "List all prefix subfolders"
)]
pub struct ListCmdConfig {
    // #[arg(
    //     short = 'e',
    //     long = "encoded",
    //     help = "encode value as base64 string",
    //     action
    // )]
    // pub is_encoded: bool,
    #[arg(help = "Target prefix")]
    pub prefix: String,
}
