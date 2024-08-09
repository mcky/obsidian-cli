use anyhow::Context;
use clap::{Args, Subcommand};
use dialoguer::Confirm;
use std::{env, fs, path::PathBuf, process};

use crate::{
    formats::yaml_to_json_value,
    util::{read_note, resolve_note_path},
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

    /// Print the Obsidian URI of a note
    Uri(UriArgs),

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
struct UriArgs {
    #[command(flatten)]
    common: NoteArgs,
}

#[derive(Args, Debug, Clone)]
struct EditArgs {
    /// create the file if it doesn't already exist
    #[arg(long, action)]
    create: bool,

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
        Some(Subcommands::Uri(UriArgs { common })) => uri(&common.note, &common.vault),
        Some(Subcommands::Open(OpenArgs { common })) => open(&common.note, &common.vault),
        Some(Subcommands::Create(CreateArgs { common })) => create(&common.note),
        Some(Subcommands::Edit(EditArgs {
            common,
            create: should_create,
        })) => edit(&common.note, should_create),
        Some(Subcommands::Path(PathArgs { common })) => path(&common.note),
        Some(Subcommands::Render(RenderArgs { common })) => render(&common.note),
        Some(Subcommands::Properties(PropertiesArgs { common, format, .. })) => {
            properties(&common.note, format)
        }
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

fn obsidian_note_uri(note_path: &PathBuf, vault: &Option<String>) -> String {
    let uri: String;

    if let Some(vault) = vault {
        uri = format!(
            "obsidian://open?vault={vault}&file={file}",
            file = note_path.display()
        );
    } else {
        uri = format!("obsidian://open?file={file}", file = note_path.display());
    }

    uri
}

fn open(note: &str, vault: &Option<String>) -> ObxResult {
    let note_path = resolve_note_path(note)?;
    let uri = obsidian_note_uri(&note_path, vault);

    open::that(&uri).with_context(|| format!("Could not open obsidian url `{uri}`"))?;

    Ok(None)
}

fn uri(note: &str, vault: &Option<String>) -> ObxResult {
    let note_path = resolve_note_path(note)?;
    let uri = obsidian_note_uri(&note_path, vault);

    Ok(Some(uri))
}

fn create(note: &str) -> ObxResult {
    let note_path = resolve_note_path(note)?;

    let note_contents = "";
    fs::write(&note_path, &note_contents)
        .with_context(|| format!("Could not create note {}", note_path.display()))?;

    Ok(None)
}

fn edit(note: &str, create_flag: &bool) -> ObxResult {
    let note_path = resolve_note_path(&note)?;

    let note_exists = note_path.exists();
    let term_is_attended = console::user_attended();

    if !note_exists {
        let mut confirmation = false;

        if term_is_attended && !create_flag {
            let prompt = format!("The note {note} does not exist, would you like to create it?");

            confirmation = Confirm::new()
                .with_prompt(prompt)
                .interact()
                .context("couldn't prompt user for confirmation to create note")?;
        }

        if confirmation || *create_flag {
            fs::File::create(&note_path).with_context(|| {
                format!("failed to create new note at {}", &note_path.display())
            })?;
        } else {
            return Ok(Some("Aborted".to_string()));
        }
    }

    let editor = env::var("EDITOR").context("$EDITOR not found")?;

    let editor_status = process::Command::new(&editor)
        .arg(&note_path)
        .status()
        .with_context(|| format!("failed to execute $EDITOR={editor}"))?;

    if editor_status.success() {
        // @TODO: this isn't strictly true, discarding changes with :q!
        // in vim will still show this message
        Ok(Some(format!("Saved changes to {}", &note_path.display())))
    } else {
        Err(anyhow::Error::msg("Editor exited with non-0 exit code"))
    }
}

fn path(_note: &str) -> ObxResult {
    todo!()
}

fn render(_note: &str) -> ObxResult {
    todo!()
}

fn properties(note: &str, format: &ExportFormatOption) -> ObxResult {
    let note_path = resolve_note_path(note)?;

    let note = read_note(&note_path).with_context(|| "could not parse note")?;

    let Some(properties) = note.properties else {
        return Ok(None);
    };

    let formatted = match format {
        ExportFormatOption::Json => {
            let as_json_value = yaml_to_json_value(&properties);
            serde_json::to_string(&as_json_value)?
        }
        ExportFormatOption::Pretty => todo!(),
        ExportFormatOption::Html => todo!(),
    };

    Ok(Some(formatted))
}

fn export(_note: &str) -> ObxResult {
    todo!()
}

fn backlinks(_note: &str) -> ObxResult {
    todo!()
}
