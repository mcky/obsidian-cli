use crate::cli_config;
use anyhow::Context;
use atty::{is, Stream};
use libobsidian::{ObsidianNote, Properties};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub type CommandResult = anyhow::Result<Option<String>>;

pub fn resolve_note_path(path_or_string: &str, vault_path: &PathBuf) -> anyhow::Result<PathBuf> {
    let file_path = Path::new(path_or_string);

    let path_with_ext: PathBuf = match file_path.extension().and_then(OsStr::to_str) {
        Some("md") => file_path.to_path_buf(),
        Some(_) => file_path.to_owned(),
        None => file_path.with_extension("md"),
    };

    let note_path = vault_path.join(path_with_ext);

    Ok(note_path)
}

pub fn get_current_vault(vault_name_override: Option<String>) -> anyhow::Result<cli_config::Vault> {
    let config = cli_config::read()?;
    let vault_name = vault_name_override.unwrap_or(config.current_vault);

    let found_vault: cli_config::Vault = config
        .vaults
        .iter()
        .find(|v| v.name == vault_name)
        .context("Expected to find the current vault in config")?
        .clone();

    Ok(found_vault)
}

pub fn should_enable_interactivity() -> bool {
    is(Stream::Stderr) || is(Stream::Stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("foo", "foo.md" ; "plain filename")]
    #[test_case("bar/foo", "bar/foo.md" ; "with path")]
    #[test_case("foo.txt", "foo.txt" ; "with another extension")]
    #[test_case("foo.md", "foo.md" ; "with markdown extension")]
    fn resolve_note_path_returns_correct_extension(input: &str, expected: &str) {
        assert_eq!(
            resolve_note_path(input, &PathBuf::from("")).unwrap(),
            PathBuf::from(expected)
        );
    }

    #[test_case("foo.md", "/path/to/", "/path/to/foo.md")]
    #[test_case("foo", "/path/to/", "/path/to/foo.md")]
    fn resolve_note_path_joins_vault_dir(file: &str, vault: &str, expected: &str) {
        assert_eq!(
            resolve_note_path(file, &PathBuf::from(vault)).unwrap(),
            PathBuf::from(expected)
        );
    }

    #[test]
    #[ignore]
    fn note_path_errors_on_invalid() {
        assert!(resolve_note_path(" ", &PathBuf::from("")).is_err());
    }
}
