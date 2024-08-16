use std::process::exit;

use clap::{Parser, Subcommand};

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
    Notes(commands::notes::NotesCommand),

    Vaults(commands::vaults::VaultsCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let res = match &cli.command {
        Some(Commands::Notes(args)) => commands::notes::entry(args),
        Some(Commands::Vaults(args)) => commands::vaults::entry(args),
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
