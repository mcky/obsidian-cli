use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::OnceLock};

fn obsidian_app_settings_path() -> &'static PathBuf {
    static SETTINGS_PATH: OnceLock<PathBuf> = OnceLock::new();

    SETTINGS_PATH.get_or_init(|| {
        #[cfg(target_os = "macos")]
        let path = etcetera::home_dir()
            .expect("should be able to find home dir")
            .join("Library/Application Support/obsidian/obsidian.json");

        #[cfg(target_os = "windows")]
        let path = PathBuf::from("%APPDATA%\\Obsidian\\obsidian.json");

        #[cfg(target_os = "linux")]
        let path = etcetera::choose_base_strategy()
            .expect("etcetera base strategy should work")
            .config_dir()
            .join("Obsidian/obsidian.json");

        path
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vault {
    pub path: PathBuf,
    pub ts: PathBuf,
    pub open: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub vaults: HashMap<String, Vault>,
}

pub fn read() -> anyhow::Result<Settings> {
    let settings_path = obsidian_app_settings_path();
    let settings_file = fs::read_to_string(settings_path).with_context(|| {
        format!(
            "failed to read obsidian app settings file at path `{}`",
            settings_path.display()
        )
    })?;

    let settings = serde_yaml::from_str(&settings_file)?;

    Ok(settings)
}
