# Obsidian CLI
Manage your obsidian vaults, notes and databases from the command line

<p align="center">
    <img src="https://github.com/user-attachments/assets/a6d8fd4c-3d1e-49d4-aadf-1226ca3a31d9" width="600" alt="Obsidian CLI Demo">
</p>

### Features
- Create, edit and read notes from the command line
- View and export properties from your notes
- Soon: query your vaults and database folders with SQL

## Commands
### Notes

```
Commands for interacting with individual notes

Usage: obx notes
       obx notes <COMMAND>

Commands:
  view        Output the raw markdown contents of a note
  open        Open a note in the Obsidian app
  uri         Print the Obsidian URI of a note
  create      Create a new note
  edit        Open a note in your default editor ($EDITOR)
  path        Print the full file-path of the note
  properties  View the properties of a note
  help        Print this message or the help of the given subcommand(s)
```

Usage:

```sh
# Print a note
> obx notes view simple-note

# Notes commands all accept a --vault specifier
> obx notes view from-another-vault --vault=secondary

> obx notes create new-note

# Edit a note in $EDITOR
> obx notes edit simple-note

# Open the note in Obsidian.app
> obx notes open simple-note

# Print the obsidian:// uri for the note
> obx notes uri simple-note

# Print the absolute path to a note
> obx notes path simple-note

# Print the properties of a note in a table
> obx notes properties with-fm-properties

# Print properties as JSON
> obx notes properties with-fm-properties -f json
```

## Vaults

```
Commands for interacting with vaults

Usage: obx vaults
       obx vaults <COMMAND>

Commands:
  create   Create a new vault and switch to it. The name will be inferred from the last segment unless --name is explicitly provided
  list     List all vaults
  switch   Set a vault as current, to be implicitly used by commands. A vault can be explicitly provided, or chosen interactively
  current  Print the name and path of the current vault
  path     Print the absolute path to the current vault
  help     Print this message or the help of the given subcommand(s)

```

```sh
# Create a new vault called "new-vault"
> obx vaults create path/to/new-vault

# Explicitly name a vault
> obx vaults create path/to/new-vault --name another-vault

# Print a table of vaults
> obx vaults list

# Print the vaults as JSON
> obx vaults list -f json

# Interactively switch vaults
> obx vaults switch

# Switch to a named vault
> obx vaults switch secondary

# Print information about the current vault
> obx vaults current

# Print the absolute path to the current vault
# Useful for combining, e.g. tree $(obx vaults path)
> obx vaults path
```


### Roadmap
- [ ] Fuzzy searching of files within vaults
- [ ] Pretty rendering of notes in the command line
  - It's been tricky finding a markdown renderer with support for all the features I'd expect, so for
    now I suggest piping to another tool such as [`glow`](https://github.com/charmbracelet/glow), e.g. `obx notes view my-note | glow`
- [ ] Query your vault with SQL
  - [ ] Query notes across a vault
  - [ ] Query a "database folder"
- [ ] Run dataview queries from the command line
