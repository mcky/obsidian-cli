use assert_fs::prelude::*;
use indoc::indoc;
mod utils;
use predicates::prelude::*;
use serde_json::json;
use utils::*;

mod notes {
    use super::*;

    #[test]
    fn accepts_full_paths() {
        Obx::from_command("notes view simple-note.md").assert_success();
    }

    #[test]
    fn accepts_without_extension() {
        Obx::from_command("notes view simple-note").assert_success();
    }

    #[test]
    fn accepts_paths() {
        Obx::from_command("notes view folder/child-note.md").assert_success();
    }

    #[test]
    fn allows_specifying_vault() {
        Obx::from_command("notes view from-another-vault --vault=secondary").assert_success();
    }

    mod view {
        use super::*;

        #[test]
        fn prints_note_markdown_content() {
            Obx::from_command("notes view simple-note.md").assert_stdout(indoc! {
            r"
                # Simple note

                This is the contents of simple-note.md
            "
            });
        }

        #[test]
        fn fails_for_missing_files() {
            Obx::from_command("notes view does-not-exist.md")
                .assert_stderr("Could not read note `does-not-exist.md`\n");
        }
    }

    mod render {
        use super::*;

        #[test]
        #[ignore = "render command not implemented"]
        fn pretty_prints_note() {
            Obx::from_command("notes render complex-note.md").assert_stdout(indoc! {r"
                ┄Rich note
                
                This is the contents of complex-note.md

                It contains a list

                • item 1
                • item 2
                • item 3

                An outbound link, and a [[simple-note |backlink]]
            "});

            Obx::from_command("notes render table.md").assert_stdout(indoc! {r"
                | Command         | Description                      |
                |-----------------|----------------------------------|
                | note view       | Print the raw markdown of a note |
                | note render     | Pretty-print a notes markdown    |
                | note properties | Print a notes properties         |
            "});
        }

        #[test]
        #[ignore = "render command not implemented"]
        fn renders_without_frontmatter() {
            Obx::from_command("notes render with-fm-properties.md").assert_stdout(indoc! {
            r"The main content of the file

            "
            });
        }

        #[test]
        #[ignore = "wikilink rewriting not implemented"]
        fn backlinks_replaced_with_clickable() {
            Obx::from_command("notes render link-types.md").assert_stdout(indoc! {
            r"
            "
            });
        }
    }

    mod create {
        use super::*;

        #[test]
        fn creates_new_note_file() {
            Obx::from_command("notes create new-note.md").assert_created("main-vault/new-note.md");
        }

        #[test]
        fn accepts_without_extension() {
            Obx::from_command("notes create new-note").assert_created("main-vault/new-note.md");
        }

        #[test]
        fn creates_missing_paths() {
            Obx::from_command("notes create some/nested/folder/new-note.md")
                .assert_created("main-vault/some/nested/folder/new-note.md");
        }

        #[test]
        fn allows_specifying_vault() {
            Obx::from_command("notes create created-in-another-vault --vault=secondary")
                .assert_created("another/path/created-in-another-vault.md");
        }

        #[test]
        #[ignore = "not implemented"]
        fn fails_if_note_exists() {
            Obx::from_command("notes create simple-note.md")
                .assert_stderr("The note simple-node.md already exists");
        }

        #[test]
        fn opens_editor() {
            let cmd = Obx::from_command("notes create new-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("main-vault/new-note.md");

            let _ = &cmd.assert_success();
            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }
    }

    mod edit {
        use super::*;

        #[test]
        fn opens_editor() {
            let cmd = Obx::from_command("notes edit simple-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("main-vault/simple-note.md");

            let _ = &cmd.assert_success();
            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }

        #[test]
        fn prints_on_editor_missing() {
            let mut cmd = Obx::from_command("notes edit simple-note.md");
            cmd.cmd.env_remove("EDITOR");

            cmd.assert_stderr("$EDITOR not found\n");
        }

        #[test]
        fn prints_on_editor_fail() {
            let cmd = Obx::from_command("notes edit simple-note.md").with_editor(r"exit 1");

            cmd.assert_stderr("Editor exited with non-0 exit code\n");
        }

        // These tests jump through some extra hoops to simulate a tty in order
        // to test interaction. Instead of calling `cmd.assert()` we spawn a bash
        // shell and assert on the lines
        //
        // When these issues are resolved we might be able to drop back to assert_cmd
        // Expose a testing API that accepts a sequence of commands and/or keystrokes to execute the prompt. #71
        // https://github.com/mikaelmello/inquire/issues/71
        // Unable to test TTY behavior #138
        // https://github.com/assert-rs/assert_cmd/issues/138
        #[test]
        fn prompts_to_create_if_missing() {
            let cmd = Obx::from_command("notes edit new-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("main-vault/new-note.md");

            let mut p = cmd.spawn_interactive(Some(5_000)).unwrap();

            p.exp_string("The note new-note.md does not exist, would you like to create it? [y/n]")
                .unwrap();

            p.send_line("y").unwrap();

            p.exp_string("Saved changes to new-note.md").unwrap();

            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }

        #[test]
        fn does_not_create_if_prompt_rejected() {
            let cmd = Obx::from_command("notes edit new-note.md");

            let edit_file = cmd.temp_dir.child("main-vault/new-note.md");

            let mut p = cmd.spawn_interactive(Some(5_000)).unwrap();

            p.exp_string("The note new-note.md does not exist, would you like to create it? [y/n]")
                .unwrap();

            p.send_line("n").unwrap();

            p.exp_string("Aborted").unwrap();

            edit_file.assert(predicate::path::missing());
        }
    }

    mod open {
        // These are wrappers around `notes uri`,
        use super::*;

        #[test]
        #[ignore = "this opens Obsidian.app on every test run"]
        fn opens_in_obsidian_with_current_vault() {
            Obx::from_command("notes open simple-note.md").assert_success();
        }

        #[test]
        #[ignore = "this opens Obsidian.app on every test run"]
        fn opens_in_obsidian_with_named_vault() {
            Obx::from_command("notes open from-another-vault.md --vault=secondary")
                .assert_success();
        }
    }

    mod uri {
        use super::*;

        #[test]
        fn prints_uri_with_current_vault() {
            let cmd = Obx::from_command("notes uri simple-note.md");
            let expected_path = cmd.temp_dir.child("main-vault/simple-note.md");
            let expected_uri = format!(
                "obsidian://open?vault=main&file={}\n",
                expected_path.display()
            );
            cmd.assert_stdout(expected_uri);
        }

        #[test]
        fn prints_uri_with_specified_vault() {
            let cmd = Obx::from_command("notes uri from-another-vault.md --vault=secondary");
            let expected_path = cmd.temp_dir.child("another/path/from-another-vault.md");
            let expected_uri = format!(
                "obsidian://open?vault=secondary&file={}\n",
                expected_path.display()
            );
            cmd.assert_stdout(expected_uri);
        }
    }

    mod path {
        use super::*;

        #[test]
        fn prints_full_path_to_file() {
            let cmd = Obx::from_command("notes path simple-note");
            let expected_path = cmd.temp_dir.child("main-vault/simple-note.md");

            cmd.assert_stdout(format!("{}\n", expected_path.display()));
        }

        #[test]
        fn allows_specifying_vault() {
            let cmd = Obx::from_command("notes path from-another-vault --vault=secondary");
            let expected_path = cmd.temp_dir.child("another/path/from-another-vault.md");

            cmd.assert_stdout(format!("{}\n", expected_path.display()));
        }
    }

    mod properties {
        use super::*;

        #[test]
        fn prints_frontmatter_properties_as_table() {
            Obx::from_command("notes properties with-fm-properties.md").assert_stdout(indoc! { r"
                ┌───────────────┬──────────────┐
                │ Property      │ Value        │
                ├───────────────┼──────────────┤
                │ test-checkbox │ true         │
                │ test-list     │ One, Two     │
                │ test-number   │ 100          │
                │ test-str      │ a string val │
                └───────────────┴──────────────┘
            " });
        }

        #[test]
        #[ignore = "file meta properties not implemented"]
        fn prints_meta_properties() {
            Obx::from_command("notes properties empty-note.md --include-meta").assert_stdout(
                indoc! { r"
                | property      | value                  |
                |---------------|------------------------|
                | path          | /path/to/empty-note.md |
                | created-at    | xxxx                   |
                " },
            );
        }

        #[test]
        fn prints_properties_as_json() {
            let stdout_match = &json!({
                "test-number": 100,
                "test-str": "a string val",
                "test-checkbox": true,
                "test-list": ["One","Two"]
            });

            Obx::from_command("notes properties with-fm-properties.md -f json")
                .assert_stdout(format!("{stdout_match}\n"));
        }

        #[test]
        fn handles_missing_frontmatter_as_json() {
            Obx::from_command("notes properties simple-note.md -f json").assert_stdout("{}\n");
        }
    }

    mod backlinks {
        use super::*;

        #[test]
        #[ignore = "backlinks not implemented"]
        fn prints_backlinks_as_table() {
            Obx::from_command("notes backlinks backlinked-to.md -f json").assert_stdout(
                indoc! { r"
                | note          | reference                  |
                |---------------|----------------------------|
                | /path/to/another-file.md | [[My backlink]] |
                " },
            );
        }

        #[test]
        #[ignore = "backlinks not implemented"]
        fn prints_backlinks_as_json() {
            Obx::from_command("notes backlinks backlinked-to.md -f json").assert_stdout(
                indoc! { r#"
                [{"note": "/path/to/another-file.md", "reference": "[[My backlink]]"}]
                "# },
            );
        }
    }

    mod export {
        use super::*;

        #[test]
        #[ignore = "export command not implemented"]
        fn exports_to_html() {
            Obx::from_command("notes export complex-note.md -f html").assert_stdout(indoc! { r"
                    <table>
                        <thead>
                            <tr>
                                <th>Property</th> 
                                <th>Value</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>some-property</td>
                                <td>a string val</td>
                            </tr>
                        </tbody>
                    </table>

                    <h1>Rich note</h1>

                    <p>This is the contents of complex-note.md</p>
                " });
        }

        #[test]
        #[ignore = "export command not implemented"]
        fn exports_to_json() {
            Obx::from_command("notes export complex-note.md -f json").assert_stdout(indoc! { r#"
                [{"note": "/path/to/another-file.md", "properties": {}, "body": ""}]
                "# });
        }
    }
}
