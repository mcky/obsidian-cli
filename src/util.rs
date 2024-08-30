use crate::{
    cli_config,
    obsidian_note::{ObsidianNote, Properties},
};
use anyhow::Context;
use atty::{is, Stream};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub type CommandResult = anyhow::Result<Option<String>>;

pub fn read_note(file_path: &PathBuf) -> anyhow::Result<ObsidianNote> {
    let file_contents = fs::read_to_string(&file_path)?;
    let note = parse_note(file_path, file_contents)?;
    Ok(note)
}

fn extract_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    let delimiter = "---";
    let mut parts = content.splitn(3, delimiter);

    match (parts.next(), parts.next(), parts.next()) {
        (Some(""), Some(frontmatter), Some(body)) => (
            Some(frontmatter.trim().to_string()),
            Some(body.trim().to_string()),
        ),
        (Some(""), Some(frontmatter), None) => (Some(frontmatter.trim().to_string()), None),
        _ => (None, Some(content.trim().to_string())),
    }
}

pub fn parse_note(file_path: &PathBuf, file_contents: String) -> anyhow::Result<ObsidianNote> {
    let (frontmatter_str, file_body) = extract_frontmatter(&file_contents);

    let frontmatter = frontmatter_str
        .map(|s| serde_yaml::from_str::<Properties>(&s))
        .transpose()?
        .and_then(|fm| {
            if fm == serde_yaml::Value::Null {
                None
            } else {
                Some(fm)
            }
        });

    let note = ObsidianNote {
        file_path: file_path.to_path_buf(),
        file_body: file_body.unwrap_or("".to_string()),
        file_contents: file_contents,
        properties: frontmatter,
    };

    Ok(note)
}

pub fn resolve_note_path(path_or_string: &str, vault_path: &PathBuf) -> anyhow::Result<PathBuf> {
    let file_path = Path::new(path_or_string);

    let path_with_ext: PathBuf = match file_path.extension().and_then(OsStr::to_str) {
        Some("md") => file_path.to_path_buf().to_owned(),
        Some(_) => file_path.to_owned(),
        None => file_path.with_extension("md"),
    };

    let note_path = vault_path.join(path_with_ext);

    return Ok(note_path);
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
    use indoc::indoc;
    use test_case::test_case;

    #[test_case("foo", "foo.md" ; "plain filename")]
    #[test_case("bar/foo", "bar/foo.md" ; "with path")]
    #[test_case("foo.txt", "foo.txt" ; "with another extension")]
    #[test_case("foo.md", "foo.md" ; "with markdown extension")]
    fn resolve_note_path_returns_correct_extension(input: &str, expected: &str) {
        assert_eq!(
            resolve_note_path(input, &PathBuf::from("")).unwrap(),
            PathBuf::from(expected)
        )
    }

    #[test_case("foo.md", "/path/to/", "/path/to/foo.md")]
    #[test_case("foo", "/path/to/", "/path/to/foo.md")]
    fn resolve_note_path_joins_vault_dir(file: &str, vault: &str, expected: &str) {
        assert_eq!(
            resolve_note_path(file, &PathBuf::from(vault)).unwrap(),
            PathBuf::from(expected)
        )
    }

    #[test]
    #[ignore]
    fn note_path_errors_on_invalid() {
        assert!(resolve_note_path(" ", &PathBuf::from("")).is_err());
    }

    #[test]
    fn parse_note_returns_body() {
        let note_content = indoc! {r#"
            ---
            some-property: foo
            ---
            The note body
        "#};
        let note = parse_note(&PathBuf::from("a-note.md"), note_content.to_string()).unwrap();

        assert_eq!(note.file_body.trim(), "The note body");
    }

    #[test]
    fn parse_note_returns_properties() {
        let note_content = indoc! {r#"
            ---
            some-property: foo
            ---
        "#};
        let note = parse_note(&PathBuf::from("a-note.md"), note_content.to_string()).unwrap();

        assert_eq!(
            note.properties,
            Some(serde_yaml::Value::Mapping(serde_yaml::Mapping::from_iter(
                vec![(
                    serde_yaml::Value::String("some-property".to_string()),
                    serde_yaml::Value::String("foo".to_string())
                )]
                .into_iter()
            )))
        );
    }

    #[test]
    fn parse_note_handles_missing_frontmatter() {
        let note =
            parse_note(&PathBuf::from("a-note.md"), "The note contents".to_string()).unwrap();
        assert_eq!(note.properties, None);
    }

    #[test]
    fn parse_note_handles_empty_frontmatter() {
        let note_content = indoc! {r#"
            ---
            ---
            The note content
        "#};

        let note = parse_note(&PathBuf::from("a-note.md"), note_content.to_string()).unwrap();
        assert_eq!(note.properties, None);
    }

    #[test]
    fn parse_note_handles_tables() {
        // Markdown tables also contain `---`
        let note_content = indoc! {r#"
            | Col1      | Col2      |
            |-----------|-----------|
            | Row1 Col1 | Row1 Col2 |
            | Row2 Col1 | Row2 Col2 |
        "#};

        let note = parse_note(&PathBuf::from("a-note.md"), note_content.to_string()).unwrap();
        assert_eq!(note.properties, None);
    }
}
