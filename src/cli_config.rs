use anyhow::Context;
use config::Config;
use etcetera::BaseStrategy;
use serde::{Deserialize, Serialize};
use std::{
    env::{self, VarError},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

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

fn get_config_dir() -> &'static PathBuf {
    static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

    CONFIG_DIR.get_or_init(|| {
        let config_dir = match env::var("OBX_CONFIG_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(VarError::NotPresent) => {
                let strategy = etcetera::choose_base_strategy().unwrap();
                strategy.config_dir()
            }
            _ => panic!("Malformed OBX_CONFIG_DIR"),
        };

        config_dir
    })
}

pub fn get_config_path() -> PathBuf {
    let config_dir = get_config_dir();
    config_dir.join("config.yml")
}

fn get_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();

    let settings = Config::builder()
        .add_source(config::File::from(config_path))
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
#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    #[cfg(target_os = "macos")]
    fn get_config_dir_returns_user_config() {
        let re = Regex::new(r"\/Users\/\w+\/.config\/").unwrap();
        let dir = format!("{}", get_config_dir().display());
        assert!(re.is_match(&dir));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn get_config_path_returns_user_config() {
        let re = Regex::new(r"\/Users\/\w+\/.config\/config.yml").unwrap();
        let dir = format!("{}", get_config_path().display());
        assert!(re.is_match(&dir));
    }
}
