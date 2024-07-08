use std::io::Result;

use clap::{Parser, Subcommand};
use rpassword::prompt_password;
use rprompt::prompt_reply;

pub mod cmd;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init { key_id: String },
    Create { name: String },
    Get { name: String },
    Edit { name: String },
}

pub fn parse() {
    match Cli::parse().command {
        Commands::Init { key_id } => cmd::init(key_id),
        Commands::Create { name } => cmd::create(name),
        Commands::Get { name } => cmd::get(name),
        Commands::Edit { name } => cmd::edit(name),
    }
}

fn prompt(question: impl ToString, mute: bool) -> Result<String> {
    if mute {
        prompt_password(question)
    } else {
        prompt_reply(question)
    }
}
