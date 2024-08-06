use anyhow::Context;
use clap::{Args, Subcommand};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
pub struct NotesCommand {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Debug, Subcommand, Clone)]
enum Subcommands {
    /// Output the raw markdown contents of a note
    View(ViewArgs),

    /// Open a note in the Obsidian app
    Open(OpenArgs),

    /// Create a new note
    Create(CreateArgs),

    /// Open a note in your default editor ($EDITOR)
    Edit(EditArgs),

    /// Print the full file-path of the note
    Path(PathArgs),

    /// Pretty-print a markdown note
    Render(RenderArgs),

    /// View the properties of a note
    Properties(PropertiesArgs),

    /// Convert the note to a range of formats
    Export(ExportArgs),

    /// View the files within the vault that contain backlinks to this file
    Backlinks(BacklinksArgs),
}

#[derive(Args, Debug, Clone)]
struct NoteArgs {
    #[arg(help = "The path to the note, if the extension is omitted .md will be assumed")]
    note: String,

    #[arg(long, short = 'v')]
    vault: Option<String>,
}

#[derive(Args, Debug, Clone)]
struct ViewArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct CreateArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct OpenArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct EditArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct PathArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct RenderArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ExportFormatOption {
    Pretty,
    Html,
    Json,
}

#[derive(Args, Debug, Clone)]
struct PropertiesArgs {
    #[arg(long, short = 'f', default_value = "pretty")]
    format: ExportFormatOption,

    #[arg(long)]
    include_meta: bool,

    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct ExportArgs {
    #[command(flatten)]
    common: NoteArgs,

    // No default, must be explicitly selected
    #[arg(long, short = 'f')]
    format: ExportFormatOption,
}

#[derive(Args, Debug, Clone)]
struct BacklinksArgs {
    #[command(flatten)]
    common: NoteArgs,

    #[arg(long, short = 'f', default_value = "pretty")]
    format: ExportFormatOption,
}

pub fn entry(cmd: &NotesCommand) -> anyhow::Result<Option<String>> {
    match &cmd.command {
        Some(Subcommands::View(ViewArgs { common })) => view(&common.note),
        Some(Subcommands::Open(OpenArgs { common })) => open(&common.note),
        Some(Subcommands::Create(CreateArgs { common })) => create(&common.note),
        Some(Subcommands::Edit(EditArgs { common })) => edit(&common.note),
        Some(Subcommands::Path(PathArgs { common })) => path(&common.note),
        Some(Subcommands::Render(RenderArgs { common })) => render(&common.note),
        Some(Subcommands::Properties(PropertiesArgs { common, .. })) => properties(&common.note),
        Some(Subcommands::Export(ExportArgs { common, .. })) => export(&common.note),
        Some(Subcommands::Backlinks(BacklinksArgs { common, .. })) => backlinks(&common.note),
        None => todo!(),
    }
}

type ObxResult = anyhow::Result<Option<String>>;

fn view(note: &str) -> ObxResult {
    let note_path = resolve_note_path(note)?;

    let note_content = fs::read_to_string(note_path.clone())
        .with_context(|| format!("Could not read note `{}`", note_path.display()))?;

    Ok(Some(note_content))
}

fn open(note: &str) -> ObxResult {
    let note_path = resolve_note_path(note)?;
    let url = format!("obsidian://open?path={}", note_path.display());

    open::that(&url).with_context(|| format!("Could not open obsidian url `{url}`"))?;

    Ok(None)
}

fn create(note: &str) -> ObxResult {
    let note_path = resolve_note_path(note)?;

    let note_contents = "";
    fs::write(&note_path, &note_contents)
        .with_context(|| format!("Could not create note {}", note_path.display()))?;

    Ok(None)
}

fn edit(note: &str) -> ObxResult {
    todo!()
}

fn path(note: &str) -> ObxResult {
    todo!()
}

fn render(note: &str) -> ObxResult {
    todo!()
}

fn properties(note: &str) -> ObxResult {
    todo!()
}

fn export(note: &str) -> ObxResult {
    todo!()
}

fn backlinks(note: &str) -> ObxResult {
    todo!()
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
