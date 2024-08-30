use crate::{
    cli_config,
    util::{get_current_vault, CommandResult},
};
use anyhow::{anyhow, Context};
use clap::{Args, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use std::{fs, io, path::PathBuf};
use tabled::{builder::Builder, settings::Style};

#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
pub struct VaultsCommand {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Debug, Subcommand, Clone)]
enum Subcommands {
    /// Create a new vault and switch to it. The name will be inferred from the last segment
    /// unless --name is explicitly provided
    Create(CreateArgs),

    /// List all vaults
    List(ListArgs),

    /// Set a vault as current, to be implicitly used by commands.
    /// A vault can be explicitly provided, or chosen interactively
    Switch(SwitchArgs),

    /// Print the name and path of the current vault
    Current,

    /// Print the absolute path to the current vault
    Path,
}

#[derive(Args, Debug, Clone)]
struct VaultArgs {}

#[derive(Args, Debug, Clone)]
struct CreateArgs {
    #[arg(help = "Path to the vault to be created")]
    vault_path: PathBuf,

    #[arg(long, help = "Explicitly name the vault")]
    name: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct SwitchArgs {
    #[arg(help = "The name of the vault to switch to")]
    vault: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ListFormats {
    Pretty,
    Json,
}

#[derive(Args, Debug, Clone)]
struct ListArgs {
    #[arg(long, short = 'f', default_value = "pretty")]
    format: ListFormats,
}

pub fn entry(cmd: &VaultsCommand) -> anyhow::Result<Option<String>> {
    match &cmd.command {
        Some(Subcommands::Create(CreateArgs {
            vault_path: vault,
            name,
        })) => create(vault, name.clone()),
        Some(Subcommands::List(ListArgs { format })) => list(format),
        Some(Subcommands::Switch(SwitchArgs { vault })) => switch(vault),
        Some(Subcommands::Current) => current(),
        Some(Subcommands::Path) => path(),
        None => todo!(),
    }
}

fn create(vault_path: &PathBuf, vault_name_override: Option<String>) -> CommandResult {
    let vault_name = vault_name_override.unwrap_or_else(|| {
        vault_path
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
    });

    let resolved_path = fs::canonicalize(vault_path).map_err(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            anyhow!(
                "Could not create vault at path `{}`, directory not found",
                vault_path.display()
            )
        } else {
            anyhow!(err)
        }
    })?;

    if !resolved_path.is_dir() {
        return Err(anyhow!(
            "Could not create vault at path `{}`, path must be a directory",
            vault_path.display()
        ));
    }

    let mut config = cli_config::read()?;

    config.current_vault = vault_name.clone();
    config.vaults.push(cli_config::Vault {
        name: vault_name.clone(),
        path: resolved_path,
    });

    let _ = cli_config::write(&config);

    Ok(Some(format!("Created vault {vault_name}")))
}

fn list(list_format: &ListFormats) -> CommandResult {
    let config = cli_config::read()?;

    let formatted = match list_format {
        &ListFormats::Json => serde_json::to_string(&config.vaults)?,
        ListFormats::Pretty => format_vault_table(&config),
    };

    Ok(Some(formatted))
}

pub fn format_vault_table(config: &cli_config::Config) -> String {
    let mut builder = Builder::new();

    for v in &config.vaults {
        builder.push_record([v.name.clone(), v.path.display().to_string()]);
    }
    builder.insert_record(0, vec!["Name", "Path"]);

    let mut table = builder.build();
    table.with(Style::sharp());

    format!("{table}")
}

fn switch(vault_name_arg: &Option<String>) -> CommandResult {
    let mut config = cli_config::read()?;

    let vault_name: String = match vault_name_arg {
        Some(s) => s.to_string(),
        None => interactive_switch(&config, "Select a vault"),
    };

    config
        .vaults
        .iter()
        .find(|v| v.name == vault_name)
        .with_context(|| {
            format!("Could not switch to vault `{vault_name}`, vault doesn't exist")
        })?;

    config.current_vault = vault_name.to_string();

    let _ = cli_config::write(&config);

    Ok(Some(format!("Switched to vault {vault_name}")))
}

pub fn interactive_switch(config: &cli_config::Config, message: &str) -> String {
    // Construct a list of vaults in the format `vault (path)`
    let vaults: Vec<String> = config
        .vaults
        .iter()
        .map(|v| format!("{} ({})", v.name.clone(), v.path.display()))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .items(&vaults)
        .interact()
        .unwrap();

    let selected_vault = &config.vaults[selection];

    selected_vault.name.to_string()
}

fn current() -> CommandResult {
    let config = cli_config::read()?;

    let found_vault = config
        .vaults
        .iter()
        .find(|v| v.name == config.current_vault)
        .context("Expected to find vault matching current_vault in config")?;

    let out = format!(
        "Current vault is `{name}` at path `{path}`",
        name = found_vault.name,
        path = found_vault.path.display()
    );

    Ok(Some(out))
}

fn path() -> CommandResult {
    let vault = get_current_vault(None)?;
    let vault_path = vault.path.to_str().unwrap().to_string();

    Ok(Some(vault_path))
}
