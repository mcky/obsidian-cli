use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;

use serde_json::json;

mod utils;
use utils::*;

mod notes {
    use super::*;

    #[test]
    fn accepts_full_paths() {
        assert_success!("notes view simple-note.md");
    }

    #[test]
    fn accepts_without_extension() {
        assert_success!("notes view simple-note");
    }

    #[test]
    fn accepts_paths() {
        assert_success!("notes view folder/child-note.md");
    }

    #[test]
    fn allows_switching_vault() {
        assert_success!("notes view from-another-vault --vault=other");
    }

    mod view {
        use super::*;

        #[test]
        fn prints_note_markdown_content() {
            assert_success!(
                "notes view simple-note.md",
                indoc! { r#"
                    # Simple note

                    This is the contents of simple-note.md
                "# }
            );
        }

        #[test]
        fn fails_for_missing_files() {
            assert_stderr!(
                "notes view does-not-exist.md",
                "Could not read note `does-not-exist.md`"
            );
        }
    }

    mod render {
        use super::*;

        #[test]
        fn pretty_prints_note() {
            assert_success!(
                "notes view simple-note.md",
                indoc! { r#"
                    Simple note pretty printed
                "# }
            );
        }
    }

    mod create {
        use super::*;

        #[test]
        fn creates_new_note_file() {
            assert_created!("notes create new-note.md", "new-note.md");
        }

        #[test]
        fn accepts_without_extension() {
            assert_created!("notes create new-note", "new-note.md");
        }

        #[test]
        fn accepts_paths() {
            assert_created!("notes create folder/new-note.md", "folder/new-note.md");
        }

        #[test]
        fn allows_switching_vault() {
            assert_created!(
                "notes create in-another-vault --vault=other",
                "other-vault/in-another-vault.md"
            );
        }

        #[test]
        fn opens_editor() {
            let (_dir, _cmd) = exec_with_fixtures("notes create new-note.md");
            assert!(false);
        }
    }

    mod edit {
        use super::*;

        #[test]
        fn opens_editor() {
            // Opens in $EDITOR
            let (_dir, _cmd) = exec_with_fixtures("notes edit simple-note.md");
            assert!(false);
        }

        #[test]
        fn prompts_to_create_if_missing() {
            let (_dir, _cmd) = exec_with_fixtures("notes edit new-note.md");
            assert!(false);
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
            assert_success!(
                "notes uri simple-note.md",
                "obsidian://open?file=simple-note.md"
            );
        }

        #[test]
        fn opens_in_obsidian_with_named_vault() {
            assert_success!(
                "notes uri simple-note.md --vault=other",
                "obsidian://open?vault=other&file=simple-note.md"
            );
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
            assert_success!(
                "notes properties with-fm-properties.md",
                indoc! { r#"
                | property      | value        |
                |---------------|--------------|
                | test-number   | 100          |
                | test-str      | a string val |
                | test-checkbox | true         |
                | test-list     | One, Two     |
                "# }
            );
        }

        #[test]
        fn prints_meta_properties() {
            assert_success!(
                "notes properties empty-note.md --include-meta",
                indoc! { r#"
                | property      | value                  |
                |---------------|------------------------|
                | path          | /path/to/empty-note.md |
                | created-at    | xxxx                   |
                "# }
            );
        }

        #[test]
        fn prints_properties_as_json() {
            assert_success!(
                "notes properties with-fm-properties.md -f json",
                json!({
                    "test-number": 100,
                    "test-str": "a string val",
                    "test-checkbox": true,
                    "test-list": ["One","Two"]
                })
                .to_string()
            );
        }
    }

    mod backlinks {
        use super::*;

        #[test]
        fn prints_backlinks_as_table() {
            assert_success!(
                "notes backlinks backlinked-to.md -f json",
                indoc! { r#"
                | note          | reference                  |
                |---------------|----------------------------|
                | /path/to/another-file.md | [[My backlink]] |
                "# }
            );
        }

        #[test]
        fn prints_backlinks_as_json() {
            assert_success!(
                "notes backlinks backlinked-to.md -f json",
                indoc! { r#"
                [{"note": "/path/to/another-file.md", "reference": "[[My backlink]]"}]
                "# }
            );
        }
    }

    mod export {
        use super::*;

        #[test]
        fn exports_to_html() {
            assert_success!(
                "notes export complex-note.md -f html",
                indoc! { r#"
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
                "# }
            );
        }

        #[test]
        fn exports_to_json() {
            assert_success!(
                "notes export complex-note.md -f json",
                indoc! { r#"
                [{"note": "/path/to/another-file.md", "properties": {}, "body": ""}]
                "# }
            );
        }
    }
}
