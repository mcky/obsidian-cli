use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
mod utils;
use predicates::prelude::*;
use serde_json::json;
use utils::*;

mod notes {
    use super::*;

    #[test]
    fn accepts_full_paths() {
        Obz::from_command("notes view simple-note.md").assert_success()
    }

    #[test]
    fn accepts_without_extension() {
        Obz::from_command("notes view simple-note").assert_success()
    }

    #[test]
    fn accepts_paths() {
        Obz::from_command("notes view folder/child-note.md").assert_success()
    }

    #[test]
    fn allows_switching_vault() {
        Obz::from_command("notes view from-another-vault --vault=other").assert_success()
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
            })
        }

        #[test]
        fn fails_for_missing_files() {
            Obz::from_command("notes view does-not-exist.md")
                .assert_stderr("Could not read note `does-not-exist.md`");
        }
    }

    mod render {
        use super::*;

        #[test]
        fn pretty_prints_note() {
            Obz::from_command("notes view simple-note.md").assert_stdout(indoc! {
            r#"
                Simple note pretty printed
            "#
            });
        }
    }

    mod create {
        use super::*;

        #[test]
        fn creates_new_note_file() {
            Obz::from_command("notes create new-note.md").assert_created("new-note.md")
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
        fn allows_switching_vault() {
            Obz::from_command("notes create in-another-vault --vault=other")
                .assert_created("other-vault/in-another-vault.md");
        }

        #[test]
        fn opens_editor() {
            let (_dir, _cmd) = exec_with_fixtures("notes create new-note.md");
            assert!(false);
        }
    }

    mod edit {
        use assert_fs::fixture::ChildPath;
        use rexpect::spawn_bash;
        use std::{fs, process};

        use super::*;

        fn create_editor(content: &str) -> (ChildPath, TempDir) {
            let temp_dir = TempDir::new().unwrap();

            // Create a mock editor script, we can verify it's been called
            // because it will append to the file
            let mock_editor = temp_dir.child("mock_editor.sh");
            let editor_content: String = format!("#!/bin/sh\n{content}");

            mock_editor.write_str(&editor_content).unwrap();

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(mock_editor.path(), fs::Permissions::from_mode(0o755)).unwrap();
            }

            #[cfg(not(unix))]
            {
                todo!();
            }

            (mock_editor, temp_dir)
        }

        #[test]
        fn opens_editor() {
            let (mock_editor, editor_tmp_dir) =
                create_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);
            let (dir, mut cmd) = exec_with_fixtures("notes edit simple-note.md");

            cmd.env("EDITOR", &mock_editor.path().to_str().unwrap());
            cmd.assert().success();

            let edit_file = dir.child("simple-note.md");

            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));

            editor_tmp_dir.close().unwrap();
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
            let (mock_editor, editor_tmp_dir) = create_editor(r#"exit 1"#);
            let (_dir, mut cmd) = exec_with_fixtures("notes edit simple-note.md");

            cmd.env("EDITOR", &mock_editor.path().to_str().unwrap());
            cmd.assert().failure().stderr(predicates::str::contains(
                "Editor exited with non-0 exit code",
            ));

            editor_tmp_dir.close().unwrap();
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
            let (mock_editor, editor_tmp_dir) =
                create_editor(r#"echo "This was appended by \$EDITOR" >> "$1""#);
            let (dir, mut cmd) = exec_with_fixtures("notes edit new-note.md");

            cmd.env("EDITOR", &mock_editor.path().to_str().unwrap());

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

            p.send_line("y").unwrap();

            p.exp_string("Saved changes to new-note.md").unwrap();

            edit_file.assert(predicate::str::contains("This was appended by $EDITOR"));

            editor_tmp_dir.close().unwrap();
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
        fn opens_in_obsidian_with_default_vault() {
            let (_dir, mut _cmd) = exec_with_fixtures("notes open simple-note.md");
            assert!(false)
        }

        #[test]
        fn opens_in_obsidian_with_named_vault() {
            let (_dir, mut _cmd) = exec_with_fixtures("notes open simple-note.md --vault=other");
            assert!(false)
        }
    }

    mod uri {
        use super::*;

        #[test]
        fn opens_in_obsidian_with_default_vault() {
            Obz::from_command("notes uri simple-note.md")
                .assert_stdout("obsidian://open?file=simple-note.md");
        }

        #[test]
        fn opens_in_obsidian_with_named_vault() {
            Obz::from_command("notes uri simple-note.md --vault=other")
                .assert_stdout("obsidian://open?vault=other&file=simple-note.md");
        }
    }

    mod path {
        use super::*;

        #[test]
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
                | property      | value        |
                |---------------|--------------|
                | test-number   | 100          |
                | test-str      | a string val |
                | test-checkbox | true         |
                | test-list     | One, Two     |
                "# });
        }

        #[test]
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
            Obz::from_command("notes properties with-fm-properties.md -f json").assert_stdout(
                json!({
                    "test-number": 100,
                    "test-str": "a string val",
                    "test-checkbox": true,
                    "test-list": ["One","Two"]
                })
                .to_string(),
            );
        }
    }

    mod backlinks {
        use super::*;

        #[test]
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
        fn exports_to_json() {
            Obz::from_command("notes export complex-note.md -f json").assert_stdout(indoc! { r#"
                [{"note": "/path/to/another-file.md", "properties": {}, "body": ""}]
                "# });
        }
    }
}
