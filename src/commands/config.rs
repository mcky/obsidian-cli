use crate::{cli_config, util::CommandResult};
use anyhow::Context;
use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
pub struct ConfigCommand {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Debug, Subcommand, Clone)]
enum Subcommands {
    /// Print the current configuration
    Print(PrintArgs),

    /// Print the absolute path to your config file
    Path,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum PrintFormats {
    Yaml,
    Json,
}

#[derive(Args, Debug, Clone)]
struct PrintArgs {
    #[arg(long, short = 'f', default_value = "yaml")]
    format: PrintFormats,
}

pub fn entry(cmd: &ConfigCommand) -> anyhow::Result<Option<String>> {
    match &cmd.command {
        Some(Subcommands::Print(PrintArgs { format })) => print(format),
        Some(Subcommands::Path) => path(),
        None => todo!(),
    }
}

fn print(format: &PrintFormats) -> CommandResult {
    let config = cli_config::read()?;

    let res = match format {
        PrintFormats::Yaml => serde_yaml::to_string(&config)?,
        PrintFormats::Json => serde_json::to_string(&config)?,
    };

    Ok(Some(res))
}

fn path() -> CommandResult {
    let config_path = cli_config::get_config_path()
        .to_str()
        .context("failed to stringify config path")?
        .to_string();

    Ok(Some(config_path))
}
