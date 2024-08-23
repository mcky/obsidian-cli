use std::process::exit;

use clap::{Parser, Subcommand};

pub mod app_settings;
pub mod cli_config;
pub mod commands;
pub mod formats;
pub mod obsidian_note;
pub mod util;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Commands for interacting with individual notes
    Notes(commands::notes::NotesCommand),

    /// Commands for interacting with vaults
    Vaults(commands::vaults::VaultsCommand),

    Init(commands::init::InitCommand),
    /// Commands for managing config
    Config(commands::config::ConfigCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let res = match &cli.command {
        Some(Commands::Notes(args)) => commands::notes::entry(args),
        Some(Commands::Vaults(args)) => commands::vaults::entry(args),
        Some(Commands::Init(args)) => commands::init::entry(args),
        Some(Commands::Config(args)) => commands::config::entry(args),
        None => {
            todo!("Needs a sub-command");
        }
    };

    match res {
        Ok(Some(content)) => {
            println!("{}", content);
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("{e}");
            exit(1)
        }
    }

    Ok(())
}
