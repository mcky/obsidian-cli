use anyhow::Context;
use config::Config;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFileVault {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub current_vault: String,
    pub vaults: Vec<ConfigFileVault>,
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

pub fn read() -> anyhow::Result<ConfigFile> {
    let config = get_config()?
        .try_deserialize::<ConfigFile>()
        .context("failed to deserialize config")?;
    Ok(config)
}

pub fn write(new_config: ConfigFile) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let serialized = serde_yaml::to_string(&new_config)?;

    fs::write(config_path, serialized)
        .with_context(|| "failed to write to config file {config_path}")
}
