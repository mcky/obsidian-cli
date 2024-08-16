use anyhow::{anyhow, Context};
use clap::{Args, Subcommand};
use config::Config;
use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};
use tabled::{builder::Builder, settings::Style};

#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
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

    /// Set a vault as current, to be implicitly used by commands
    Switch(SwitchArgs),

    /// Print the name and path of the current vault
    Current,
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
    #[arg(help = "The identifier of the vault to be unlinked")]
    vault: String,
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
        })) => create(&vault, name.clone()),
        Some(Subcommands::List(ListArgs { format })) => list(format),
        Some(Subcommands::Switch(SwitchArgs { vault })) => switch(&vault),
        Some(Subcommands::Current) => current(),
        None => todo!(),
    }
}

type ObxResult = anyhow::Result<Option<String>>;

#[derive(Serialize, Deserialize, Debug)]
struct ConfigFileVault {
    name: String,
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigFile {
    current_vault: String,
    vaults: Vec<ConfigFileVault>,
}

static DEFAULT_CONFIG_PATH: &str = "SHOULD_ERROR.yml";

fn get_config_path() -> String {
    env::var("OBZ_CONFIG").unwrap_or(DEFAULT_CONFIG_PATH.to_string())
}

fn get_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();

    let settings = Config::builder()
        .add_source(config::File::with_name(&config_path))
        .build()?;

    Ok(settings)
}

fn read_config() -> anyhow::Result<ConfigFile> {
    let config = get_config()?
        .try_deserialize::<ConfigFile>()
        .context("failed to deserialize config")?;
    Ok(config)
}

fn write_config(new_config: ConfigFile) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let serialized = serde_yaml::to_string(&new_config)?;

    fs::write(config_path, serialized)
        .with_context(|| "failed to write to config file {config_path}")
}

fn create(vault_path: &PathBuf, vault_name_override: Option<String>) -> ObxResult {
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

    let mut cfg = read_config()?;

    cfg.vaults.push(ConfigFileVault {
        name: vault_name.clone(),
        path: resolved_path.to_path_buf(),
    });

    let _ = write_config(cfg);

    Ok(Some(format!("Created vault {vault_name}")))
}

fn list(list_format: &ListFormats) -> ObxResult {
    let config = read_config()?;

    let formatted = match list_format {
        &ListFormats::Json => {
            let json = serde_json::to_string(&config.vaults)?;
            json
        }
        ListFormats::Pretty => {
            let mut builder = Builder::new();

            for v in config.vaults {
                builder.push_record([v.name, v.path.display().to_string()])
            }
            builder.insert_record(0, vec!["Name", "Path"]);

            let mut table = builder.build();
            table.with(Style::sharp());

            format!("{table}")
        }
    };

    Ok(Some(formatted))
}

fn switch(vault_name: &str) -> ObxResult {
    let config = read_config()?;

    let found_vault = config
        .vaults
        .iter()
        .find(|v| v.name == vault_name)
        .with_context(|| {
            format!("Could not switch to vault `{vault_name}`, vault doesn't exist")
        })?;

    dbg!(found_vault);

    let mut cfg = read_config()?;
    cfg.current_vault = vault_name.to_string();

    let _ = write_config(cfg);

    Ok(Some(format!("Switched to {vault_name}")))
}

fn current() -> ObxResult {
    let config = read_config()?;

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
