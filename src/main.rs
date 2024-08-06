use std::process::exit;

use clap::{Parser, Subcommand};

pub mod commands;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Notes(commands::notes::NotesCommand),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let res = match &cli.command {
        Some(Commands::Notes(args)) => commands::notes::entry(args),
        Some(cmd) => {
            todo!("command {cmd:?}");
        }
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
