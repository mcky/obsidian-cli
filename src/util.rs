use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use crate::obsidian_note::{ObsidianNote, Properties};

pub fn read_note(file_path: &PathBuf) -> anyhow::Result<ObsidianNote> {
    let file_contents = fs::read_to_string(&file_path)?;
    let note = parse_note(file_path, file_contents)?;
    Ok(note)
}

fn extract_frontmatter(content: &str) -> (Option<String>, String) {
    let delimiter = "---";
    let mut parts = content.splitn(3, delimiter);

    match (parts.next(), parts.next(), parts.next()) {
        (Some(""), Some(frontmatter), Some(body)) => (
            Some(frontmatter.trim().to_string()),
            body.trim().to_string(),
        ),
        (Some(""), Some(frontmatter), None) => {
            (Some(frontmatter.trim().to_string()), "".to_string())
        }
        (Some(body), None, None) => (None, body.trim().to_string()),
        _ => panic!("Failed to parse frontmatter"),
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
        file_body: file_body,
        file_contents: file_contents,
        properties: frontmatter,
    };

    Ok(note)
}

pub fn resolve_note_path(path_or_string: &str) -> anyhow::Result<PathBuf> {
    let file_path = Path::new(path_or_string);

    let content_type: PathBuf = match file_path.extension().and_then(OsStr::to_str) {
        Some("md") => file_path.to_path_buf().to_owned(),
        Some(_) => file_path.to_owned(),
        None => file_path.with_extension("md"),
    };

    return Ok(content_type);
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
    fn note_path_resolves(input: &str, expected: &str) {
        assert_eq!(resolve_note_path(input).unwrap(), PathBuf::from(expected))
    }

    #[test]
    #[ignore]
    fn note_path_errors_on_invalid() {
        assert!(resolve_note_path(" ").is_err());
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
}
