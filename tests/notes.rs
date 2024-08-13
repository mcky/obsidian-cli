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
        Obz::from_command("notes view simple-note.md").assert_success();
    }

    #[test]
    fn accepts_without_extension() {
        Obz::from_command("notes view simple-note").assert_success();
    }

    #[test]
    fn accepts_paths() {
        Obz::from_command("notes view folder/child-note.md").assert_success();
    }

    #[test]
    #[ignore = "vault handling not implemented"]
    fn allows_switching_vault() {
        Obz::from_command("notes view from-another-vault --vault=other").assert_success();
    }

    mod view {
        use super::*;

        #[test]
        fn prints_note_markdown_content() {
            Obz::from_command("notes view simple-note.md").assert_stdout(indoc! {
            r#"
                # Simple note

                This is the contents of simple-note.md
            "#
            });
        }

        #[test]
        fn fails_for_missing_files() {
            Obz::from_command("notes view does-not-exist.md")
                .assert_stderr("Could not read note `does-not-exist.md`\n");
        }
    }

    mod render {
        use super::*;

        #[test]
        #[ignore = "render command not implemented"]
        fn pretty_prints_note() {
            Obz::from_command("notes render complex-note.md").assert_stdout(indoc! {r#"
                ┄Rich note
                
                This is the contents of complex-note.md

                It contains a list

                • item 1
                • item 2
                • item 3

                An outbound link, and a [[simple-note |backlink]]
            "#});

            Obz::from_command("notes render table.md").assert_stdout(indoc! {r#"
                | Command         | Description                      |
                |-----------------|----------------------------------|
                | note view       | Print the raw markdown of a note |
                | note render     | Pretty-print a notes markdown    |
                | note properties | Print a notes properties         |
            "#});
        }

        #[test]
        #[ignore = "render command not implemented"]
        fn renders_without_frontmatter() {
            Obz::from_command("notes render with-fm-properties.md").assert_stdout(indoc! {
            r#"The main content of the file

            "#
            });
        }

        #[test]
        #[ignore = "wikilink rewriting not implemented"]
        fn backlinks_replaced_with_clickable() {
            Obz::from_command("notes render link-types.md").assert_stdout(indoc! {
            r#"
            "#
            });
        }
    }

    mod create {
        use super::*;

        #[test]
        fn creates_new_note_file() {
            Obz::from_command("notes create new-note.md").assert_created("new-note.md");
        }

        #[test]
        fn accepts_without_extension() {
            Obz::from_command("notes create new-note").assert_created("new-note.md");
        }

        #[test]
        fn accepts_paths() {
            Obz::from_command("notes create folder/new-note.md")
                .assert_created("folder/new-note.md");
        }

        #[test]
        #[ignore = "vault handling not implemented"]
        fn allows_switching_vault() {
            Obz::from_command("notes create in-another-vault --vault=other")
                .assert_created("other-vault/in-another-vault.md");
        }

        #[test]
        #[ignore = "not implemented"]
        fn fails_if_note_exists() {
            Obz::from_command("notes create simple-note.md")
                .assert_stderr("The note simple-node.md already exists");
        }

        #[test]
        fn opens_editor() {
            let cmd = Obz::from_command("notes create new-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("new-note.md");

            let _ = &cmd.assert_success();
            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }
    }

    mod edit {
        use super::*;
        use rexpect::spawn_bash;
        use std::process;

        #[test]
        fn opens_editor() {
            let cmd = Obz::from_command("notes edit simple-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("simple-note.md");

            let _ = &cmd.assert_success();
            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }

        #[test]
        fn prints_on_editor_missing() {
            let (_dir, mut cmd) = exec_with_fixtures("notes edit simple-note.md");

            cmd.env_clear();
            cmd.assert()
                .failure()
                .stderr(predicates::str::contains("$EDITOR not found"));
        }

        #[test]
        fn prints_on_editor_fail() {
            let cmd = Obz::from_command("notes edit simple-note.md").with_editor(r#"exit 1"#);

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
            let cmd = Obz::from_command("notes edit new-note.md")
                .with_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);

            let edit_file = &cmd.temp_dir.child("new-note.md");

            // Take our usual cmd but instead of asserting on it, convert it into
            // a string, then split it into the `cd $dir` and `cmd $args` parts
            let cmd_str = &format!("{:?}", process::Command::from(cmd.cmd));
            let cmd_parts = cmd_str.split(" && ").collect::<Vec<&str>>();
            let [cd_cmd, bin_cmd] = &cmd_parts[..] else {
                panic!("couldn't split cmd_parts")
            };

            let mut p = spawn_bash(Some(1_000)).unwrap();

            p.send_line(&cd_cmd).unwrap();
            p.send_line(&bin_cmd).unwrap();

            p.exp_string("The note new-note.md does not exist, would you like to create it? [y/n]")
                .unwrap();

            p.send_line("y").unwrap();

            p.exp_string("Saved changes to new-note.md").unwrap();

            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));
        }

        #[test]
        fn does_not_create_if_prompt_rejected() {
            let (dir, cmd) = exec_with_fixtures("notes edit new-note.md");

            let edit_file = dir.child("new-note.md");

            // Take our usual cmd but instead of asserting on it, convert it into
            // a string, then split it into the `cd $dir` and `cmd $args` parts
            let cmd_str = &format!("{:?}", process::Command::from(cmd));
            let cmd_parts = cmd_str.split(" && ").collect::<Vec<&str>>();
            let [cd_cmd, bin_cmd] = &cmd_parts[..] else {
                panic!("couldn't split cmd_parts")
            };

            let mut p = spawn_bash(Some(1_000)).unwrap();

            p.send_line(&cd_cmd).unwrap();
            p.send_line(&bin_cmd).unwrap();

            p.exp_string("The note new-note.md does not exist, would you like to create it? [y/n]")
                .unwrap();

            p.send_line("n").unwrap();

            p.exp_string("Aborted").unwrap();

            edit_file.assert(predicate::path::missing());
        }
    }

    mod open {
        use super::*;

        #[test]
        #[ignore = "vault handling not implemented"]
        fn opens_in_obsidian_with_default_vault() {
            let (_dir, mut _cmd) = exec_with_fixtures("notes open simple-note.md");
            assert!(false)
        }

        #[test]
        #[ignore = "vault handling not implemented"]
        fn opens_in_obsidian_with_named_vault() {
            let (_dir, mut _cmd) = exec_with_fixtures("notes open simple-note.md --vault=other");
            assert!(false)
        }
    }

    mod uri {
        use super::*;

        #[test]
        #[ignore = "vault handling not implemented"]
        fn opens_in_obsidian_with_default_vault() {
            Obz::from_command("notes uri simple-note.md")
                .assert_stdout("obsidian://open?file=simple-note.md");
        }

        #[test]
        #[ignore = "vault handling not implemented"]
        fn opens_in_obsidian_with_named_vault() {
            Obz::from_command("notes uri simple-note.md --vault=other")
                .assert_stdout("obsidian://open?vault=other&file=simple-note.md");
        }
    }

    mod path {
        use super::*;

        #[test]
        #[ignore = "vault handling not implemented"]
        fn prints_full_path_to_file() {
            let (dir, mut cmd) = exec_with_fixtures("notes path simple-note");

            let expected_path = dir.join("/simple-note.md").to_str().unwrap().to_string();

            cmd.assert()
                .success()
                .stdout(predicate::str::contains(&expected_path));
        }
    }

    mod properties {
        use super::*;

        #[test]
        fn prints_frontmatter_properties_as_table() {
            Obz::from_command("notes properties with-fm-properties.md").assert_stdout(indoc! { r#"

                ┌───────────────┬──────────────┐
                │ Property      │ Value        │
                ├───────────────┼──────────────┤
                │ test-checkbox │ true         │
                │ test-list     │ One, Two     │
                │ test-number   │ 100          │
                │ test-str      │ a string val │
                └───────────────┴──────────────┘

            "# });
        }

        #[test]
        #[ignore = "file meta properties not implemented"]
        fn prints_meta_properties() {
            Obz::from_command("notes properties empty-note.md --include-meta").assert_stdout(
                indoc! { r#"
                | property      | value                  |
                |---------------|------------------------|
                | path          | /path/to/empty-note.md |
                | created-at    | xxxx                   |
                "# },
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

            Obz::from_command("notes properties with-fm-properties.md -f json")
                .assert_stdout(format!("{stdout_match}\n"));
        }

        #[test]
        fn handles_missing_frontmatter_as_json() {
            Obz::from_command("notes properties simple-note.md -f json").assert_stdout("{}\n");
        }
    }

    mod backlinks {
        use super::*;

        #[test]
        #[ignore = "backlinks not implemented"]
        fn prints_backlinks_as_table() {
            Obz::from_command("notes backlinks backlinked-to.md -f json").assert_stdout(
                indoc! { r#"
                | note          | reference                  |
                |---------------|----------------------------|
                | /path/to/another-file.md | [[My backlink]] |
                "# },
            );
        }

        #[test]
        #[ignore = "backlinks not implemented"]
        fn prints_backlinks_as_json() {
            Obz::from_command("notes backlinks backlinked-to.md -f json").assert_stdout(
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
            Obz::from_command("notes export complex-note.md -f html").assert_stdout(indoc! { r#"
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
                "# });
        }

        #[test]
        #[ignore = "export command not implemented"]
        fn exports_to_json() {
            Obz::from_command("notes export complex-note.md -f json").assert_stdout(indoc! { r#"
                [{"note": "/path/to/another-file.md", "properties": {}, "body": ""}]
                "# });
        }
    }
}
