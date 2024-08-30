use crate::{
    cli_config,
    formats::{yaml_to_json_value, yaml_to_string_map},
    util::{
        get_current_vault, read_note, resolve_note_path, should_enable_interactivity, CommandResult,
    },
};
use anyhow::{anyhow, Context};
use atty::{isnt, Stream};
use clap::{Args, Subcommand};
use dialoguer::Confirm;
use std::{env, fs, io, path::PathBuf, process};
use tabled::{builder::Builder, settings::Style};

#[derive(Args, Debug, Clone)]
#[command(args_conflicts_with_subcommands = true)]
#[command(arg_required_else_help = true)]
pub struct NotesCommand {
    #[command(subcommand)]
    command: Option<Subcommands>,

    #[arg(long, short = 'v', global = true)]
    vault: Option<String>,
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

    // Pretty-print a markdown note
    // Render(RenderArgs),
    /// View the properties of a note
    Properties(PropertiesArgs),
    // Convert the note to a range of formats
    // Export(ExportArgs),

    // View the files within the vault that contain backlinks to this file
    // Backlinks(BacklinksArgs),
}

#[derive(Args, Debug, Clone)]
struct NoteArgs {
    #[arg(help = "The path to the note, if the extension is omitted .md will be assumed")]
    note: String,

    #[arg(long, short = 'v', global = true)]
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
        Some(Subcommands::View(ViewArgs { common })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            view(args)
        }
        Some(Subcommands::Uri(UriArgs { common })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            uri(args)
        }
        Some(Subcommands::Open(OpenArgs { common })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            open(args)
        }
        Some(Subcommands::Create(CreateArgs { common })) => {
            let stdin = maybe_stdin()?;
            let args = EnrichedNoteArgs::from_args(common)?;
            create(args, stdin)
        }
        Some(Subcommands::Edit(EditArgs {
            common,
            create: should_create,
        })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            edit(args, should_create)
        }
        Some(Subcommands::Path(PathArgs { common })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            path(args)
        }
        // Some(Subcommands::Render(RenderArgs { common })) => {
        //     let args = EnrichedNoteArgs::from_args(common)?;
        //     render(args)
        // }
        Some(Subcommands::Properties(PropertiesArgs { common, format, .. })) => {
            let args = EnrichedNoteArgs::from_args(common)?;
            properties(args, format)
        }
        // Some(Subcommands::Export(ExportArgs { common, .. })) => {
        //     let args = EnrichedNoteArgs::from_args(common)?;
        //     export(args)
        // }
        // Some(Subcommands::Backlinks(BacklinksArgs { common, .. })) => {
        //     let args = EnrichedNoteArgs::from_args(common)?;
        //     backlinks(args)
        // }
        None => todo!(),
    }
}

struct EnrichedNoteArgs {
    vault: cli_config::Vault,
    note_path: PathBuf,
    note_file: String,
}

impl EnrichedNoteArgs {
    fn from_args(args: &NoteArgs) -> anyhow::Result<Self> {
        let vault_name = &args.vault;
        let vault = get_current_vault(vault_name.clone())?;

        let note_path = resolve_note_path(&args.note, &vault.path)?;
        let note_file = note_path
            .file_name()
            .expect("note_path should be a file")
            .to_str()
            .unwrap()
            .to_owned();

        Ok(Self {
            vault,
            note_path,
            note_file,
        })
    }
}

fn view(note: EnrichedNoteArgs) -> CommandResult {
    let note_content = fs::read_to_string(note.note_path.clone())
        .with_context(|| format!("Could not read note `{}`", note.note_file))?;

    Ok(Some(note_content))
}

fn obsidian_note_uri(note_path: &PathBuf, vault: String) -> String {
    format!(
        "obsidian://open?vault={vault}&file={file}",
        file = note_path.display()
    )
}

fn open(note: EnrichedNoteArgs) -> CommandResult {
    let uri = obsidian_note_uri(&note.note_path, note.vault.name);

    open::that(&uri).with_context(|| format!("Could not open obsidian url `{uri}`"))?;

    Ok(None)
}

fn uri(note: EnrichedNoteArgs) -> CommandResult {
    let uri = obsidian_note_uri(&note.note_path, note.vault.name);

    Ok(Some(uri))
}

fn create_note(note: &EnrichedNoteArgs, note_contents: &str) -> anyhow::Result<()> {
    // Ensure the directory exists for a provided note
    // before we try to write to it
    let note_dir = &note
        .note_path
        .parent()
        .expect("note_path should have a parent");

    fs::create_dir_all(note_dir)
        .with_context(|| format!("Could not create directory {}", note_dir.display()))?;

    fs::write(&note.note_path, note_contents)
        .with_context(|| format!("Could not create note {}", note.note_path.display()))?;

    Ok(())
}

fn create(note: EnrichedNoteArgs, stdin: Option<String>) -> CommandResult {
    let note_input: String = stdin.unwrap_or_default();

    let initial_note_content = initial_note_content_from_stdin(note_input)?;

    create_note(&note, &initial_note_content)?;

    let editor = env::var("EDITOR").context("$EDITOR not found")?;

    let editor_status = process::Command::new(&editor)
        .arg(&note.note_path)
        .status()
        .with_context(|| format!("failed to execute $EDITOR={editor}"))?;

    if editor_status.success() {
        // @TODO: this isn't strictly true, discarding changes with :q!
        // in vim will still show this message
        Ok(Some(format!("Created note {}", &note.note_path.display())))
    } else {
        Err(anyhow::Error::msg("Editor exited with non-0 exit code"))
    }
}

fn initial_note_content_from_stdin(stdin: String) -> anyhow::Result<String> {
    let trimmed = stdin.trim();

    // Check if the trimmed input looks like JSON
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(json) => {
                let yaml = serde_yaml::to_string(&json)?;
                let as_frontmatter = format!("---\n{}\n---\n", yaml.trim());
                Ok(as_frontmatter)
            }
            Err(e) => Err(anyhow!("Input looks like JSON but failed to parse: {}", e)),
        }
    } else {
        Ok(stdin)
    }
}

fn edit(note: EnrichedNoteArgs, create_flag: &bool) -> CommandResult {
    let note_exists = note.note_path.exists();
    let term_is_attended = should_enable_interactivity();

    if !note_exists {
        let mut confirmation = false;

        if term_is_attended && !create_flag {
            let prompt = format!(
                "The note {} does not exist, would you like to create it?",
                note.note_file
            );

            confirmation = Confirm::new()
                .with_prompt(prompt)
                .interact()
                .context("couldn't prompt user for confirmation to create note")?;
        }

        if confirmation || *create_flag {
            let note_contents = "";
            create_note(&note, note_contents)?;
        } else {
            return Ok(Some("Aborted".to_string()));
        }
    }

    let editor = env::var("EDITOR").context("$EDITOR not found")?;

    let editor_status = process::Command::new(&editor)
        .arg(&note.note_path)
        .status()
        .with_context(|| format!("failed to execute $EDITOR={editor}"))?;

    if editor_status.success() {
        // @TODO: this isn't strictly true, discarding changes with :q!
        // in vim will still show this message
        Ok(Some(format!("Saved changes to {}", &note.note_file)))
    } else {
        Err(anyhow::Error::msg("Editor exited with non-0 exit code"))
    }
}

fn path(note: EnrichedNoteArgs) -> CommandResult {
    let note_path = note.note_path.to_str().unwrap().to_string();
    Ok(Some(note_path))
}

fn properties(note: EnrichedNoteArgs, format: &ExportFormatOption) -> CommandResult {
    let note = read_note(&note.note_path).with_context(|| "could not parse note")?;

    let formatted = match format {
        ExportFormatOption::Json => {
            let json_value = note
                .properties
                .map(|yaml| yaml_to_json_value(&yaml))
                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
            serde_json::to_string(&json_value)?
        }
        ExportFormatOption::Pretty => {
            let Some(serde_yaml::Value::Mapping(p)) = note.properties else {
                panic!("Expected note.properties to be yaml::Value::mapping")
            };

            let mut property_strings = yaml_to_string_map(&p)
                .into_iter()
                .map(|(k, v)| vec![k, v])
                .collect::<Vec<Vec<String>>>();

            property_strings.sort();
            let sorted_properties = property_strings.iter();

            let mut builder = Builder::from_iter(sorted_properties);
            builder.insert_record(0, vec!["Property", "Value"]);

            let mut table = builder.build();
            table.with(Style::sharp());

            format!("{table}")
        }
        ExportFormatOption::Html => todo!(),
    };

    Ok(Some(formatted))
}

fn maybe_stdin() -> anyhow::Result<Option<String>> {
    match isnt(Stream::Stdin) {
        true => {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            Ok(Some(buffer))
        }
        false => Ok(None),
    }
}
