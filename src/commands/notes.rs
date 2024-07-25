use anyhow::Context;
use clap::{Args, Subcommand};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Args, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct NotesCommand {
    #[command(subcommand)]
    command: Option<Subcommands>,
    // #[command(flatten)]
    // list: FlatArgs,
}

#[derive(Debug, Subcommand, Clone)]
enum Subcommands {
    /// Output the raw markdown contents of a note
    View(ViewArgs),
}

#[derive(Debug, Args, Clone)]
struct ViewArgs {
    #[arg(help = "The path to the note, if the extension is omitted .md will be assumed")]
    note: String,
}

pub fn entry(cmd: &NotesCommand) -> anyhow::Result<()> {
    match &cmd.command {
        Some(Subcommands::View(ViewArgs { note })) => view(note),
        None => todo!(),
        _ => {
            todo!();
        }
    }
}

fn resolve_note_path(path_or_string: &str) -> anyhow::Result<PathBuf> {
    let file_path = Path::new(path_or_string);

    let content_type: PathBuf = match file_path.extension().and_then(OsStr::to_str) {
        Some("md") => file_path.to_path_buf().to_owned(),
        Some(_) => file_path.to_owned(),
        None => file_path.with_extension("md"),
    };

    return Ok(content_type);
}

fn view(note: &str) -> anyhow::Result<()> {
    let note_path = resolve_note_path(note)?;

    let note_content = fs::read_to_string(note_path.clone())
        .with_context(|| format!("could not read file `{}`", note_path.display()))?;

    println!("{note_content}");

    Ok(())
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
