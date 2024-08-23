# Obsidian CLI
Manage your obsidian vaults, notes and databases from the command line

<p align="center">
    <img src="https://github.com/user-attachments/assets/a6d8fd4c-3d1e-49d4-aadf-1226ca3a31d9" width="600" alt="Obsidian CLI Demo">
</p>

### Features
- Create, edit and read notes from the command line
- View and export properties from your notes
- Soon: query your vaults and database folders with SQL


### Roadmap
- [ ] Fuzzy searching of files within vaults
- [ ] Pretty rendering of notes in the command line
  - It's been tricky finding a markdown renderer with support for all the features I'd expect, so for
    now I suggest piping to another tool such as [`glow`](https://github.com/charmbracelet/glow), e.g. `obx notes view my-note | glow`
- [ ] Query your vault with SQL
  - [ ] Query notes across a vault
  - [ ] Query a "database folder"
- [ ] Run dataview queries from the command line
