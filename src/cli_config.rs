use crate::app_settings;
use anyhow::{bail, Context};
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

    CONFIG_DIR.get_or_init(|| match env::var("OBX_CONFIG_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(VarError::NotPresent) => {
            let strategy =
                etcetera::choose_base_strategy().expect("etcetera base strategy should work");
            strategy.config_dir().join("obx")
        }
        _ => panic!("Malformed OBX_CONFIG_DIR"),
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

pub fn exists() -> bool {
    let config_path = get_config_path();
    Path::exists(&config_path)
}

pub fn write(new_config: &File) -> anyhow::Result<()> {
    let config_path = get_config_path();
    let serialized = serde_yaml::to_string(new_config)?;

    fs::write(&config_path, serialized)
        .with_context(|| format!("failed to write to config file {}", config_path.display()))
}

impl TryFrom<app_settings::Settings> for File {
    type Error = anyhow::Error;

    fn try_from(settings: app_settings::Settings) -> Result<Self, Self::Error> {
        let vaults: Vec<Vault> = Vec::from_iter(settings.vaults.values().map(|vault| {
            Vault {
                // @TODO: fn to get name from path
                name: vault
                    .path
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string(),
                path: PathBuf::from(&vault.path),
            }
        }));

        match vaults.len() {
            0 => {
                // We can't set a current vault without having at least one
                // in future if cfg.current_vault is set to optional we
                // could remove this
                bail!("Settings must contain at least one vault")
            }
            _n => {
                let config = Self {
                    current_vault: vaults[0].clone().name,
                    vaults,
                };

                Ok(config)
            }
        }
    }
}

pub fn create_from_settings() -> anyhow::Result<File> {
    let config_dir = get_config_dir();
    let config_path = get_config_path();

    fs::create_dir_all(config_dir)?;

    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&config_path)
        .with_context(|| format!("failed to create config file at {}", config_path.display()))?;

    let settings = app_settings::read()?;
    let config = File::try_from(settings)?;

    write(&config)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    #[cfg(target_os = "macos")]
    fn get_config_dir_returns_user_config() {
        let re = Regex::new(r"\/Users\/\w+\/.config\/obx\/").unwrap();
        let dir = format!("{}", get_config_dir().display());
        assert!(re.is_match(&dir));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn get_config_path_returns_user_config() {
        let re = Regex::new(r"\/Users\/\w+\/.config\/obx\/config.yml").unwrap();
        let dir = format!("{}", get_config_path().display());
        assert!(re.is_match(&dir));
    }
}
