use anyhow::Context;
use config::Config;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vault {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub current_vault: String,
    pub vaults: Vec<Vault>,
}

static DEFAULT_CONFIG_DIR: &str = "~/.config/obx-cli";

fn get_config_dir() -> String {
    env::var("OBX_CONFIG_DIR").unwrap_or(DEFAULT_CONFIG_DIR.to_string())
}

fn get_config_path() -> String {
    PathBuf::from(get_config_dir())
        .join("./config.yml")
        .to_str()
        .unwrap()
        .to_string()
}

fn get_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();

    let settings = Config::builder()
        .add_source(config::File::with_name(&config_path))
        .build()?;

    Ok(settings)
}

pub fn read() -> anyhow::Result<File> {
    let config = get_config()?
        .try_deserialize::<File>()
        .context("failed to deserialize config")?;
    Ok(config)
}

pub fn write(new_config: File) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let serialized = serde_yaml::to_string(&new_config)?;

    fs::write(&config_path, serialized)
        .with_context(|| format!("failed to write to config file {config_path}"))
}
