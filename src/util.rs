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

pub fn parse_note(file_path: &PathBuf, file_contents: String) -> anyhow::Result<ObsidianNote> {
    let (frontmatter, file_body) =
        serde_frontmatter::deserialize::<Properties>(&file_contents).unwrap();

    let note = ObsidianNote {
        file_path: file_path.to_path_buf(),
        file_body: file_body,
        file_contents: file_contents,
        properties: Some(frontmatter),
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
}
