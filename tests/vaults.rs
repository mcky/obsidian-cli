use indoc::indoc;
mod utils;
use assert_fs::prelude::PathChild;
use serde_json::json;
use utils::*;

mod vaults {
    use super::*;

    mod create {
        use super::*;
        use assert_fs::prelude::FileTouch;

        #[test]
        fn defaults_to_folder_name() {
            let cmd = Obz::from_command("vaults create path/to/new-vault");
            let _ = &cmd.temp_dir.child("path/to/new-vault/file.md").touch();

            cmd.assert_stdout("Created vault new-vault\n");
        }

        #[test]
        fn accepts_name_param() {
            let cmd = Obz::from_command("vaults create path/to/new-vault --name another-vault");
            let _ = &cmd.temp_dir.child("path/to/new-vault/file.md").touch();

            cmd.assert_stdout("Created vault another-vault\n");
        }

        #[test]
        fn fails_on_missing_dir() {
            Obz::from_command("vaults create /does/not/exist").assert_stderr(
                "Could not create vault at path `/does/not/exist`, directory not found\n",
            );
        }

        #[test]
        fn fails_if_not_dir() {
            Obz::from_command("vaults create ./main-vault/simple-note.md").assert_stderr(
                "Could not create vault at path `./main-vault/simple-note.md`, path must be a directory\n",
            );
        }

        #[test]
        fn persists_changes() {
            let create_cmd = Obz::from_command("vaults create path/to/new/vault");
            let _ = create_cmd
                .temp_dir
                .child("path/to/new/vault/file.md")
                .touch();

            let expected_vault_path = create_cmd.temp_dir.child("path/to/new/vault");

            // Ensure list_cmd is reading from the same temp_dir as create_cmd, so we
            // pick up the persisted changes
            let mut list_cmd = Obz::from_command("vaults list");
            let tmp_config_path = create_cmd.temp_dir.child("./cfg/obz.yml");
            list_cmd.env("OBZ_CONFIG", tmp_config_path.display().to_string());

            // Keep a reference to the Obz instance so we don't drop the tmp dir
            let _ = &create_cmd.assert_success();

            let expected_path = format!("{}", expected_vault_path.display());

            list_cmd.assert_stdout_contains(expected_path);
        }

    }

    mod list {
        use super::*;

        #[test]
        fn prints_vault_list() {
            let cmd = Obz::from_command("vaults list").with_config_file(indoc! {
                r#"
                current_vault: some-vault
                vaults:
                    - name: some-vault
                      path: /some/path
            "#
            });

            cmd.assert_stdout(indoc! {"
              ┌────────────┬────────────┐
              │ Name       │ Path       │
              ├────────────┼────────────┤
              │ some-vault │ /some/path │
              └────────────┴────────────┘
            "});
        }

        #[test]
        fn prints_vault_list_as_json() {
            let cmd = Obz::from_command("vaults list -f json").with_config_file(indoc! {
                r#"
                current_vault: some-vault
                vaults:
                    - name: some-vault
                      path: /some/path
            "#
            });

            let stdout_match = &json!([
                {"name": "some-vault", "path": "/some/path"},
            ]);

            cmd.assert_stdout(format!("{stdout_match}\n"));
        }
    }

    mod switch {
        use super::*;

        #[test]
        fn prints_success_message() {
            Obz::from_command("vaults switch second-vault")
                .assert_stdout("Switched to second-vault\n");
        }

        #[test]
        fn fails_on_missing_vault() {
            Obz::from_command("vaults switch does-not-exist")
                .assert_stderr("Could not switch to vault `does-not-exist`, vault doesn't exist\n");
        }

        #[test]
        fn persists_changes() {
            let switch_cmd = Obz::from_command("vaults switch second-vault");

            // Ensure curr_cmd is reading from the same temp_dir as switch_cmd, so we
            // pick up the persisted changes
            let mut curr_cmd = Obz::from_command("vaults current");
            let tmp_config_path = switch_cmd.temp_dir.child("./cfg/obz.yml");
            curr_cmd.env("OBZ_CONFIG", tmp_config_path.display().to_string());

            // Keep a reference to the Obz instance so we don't drop the tmp dir
            let _x = switch_cmd.assert_success();

            curr_cmd.assert_stdout_contains("second-vault");
        }
    }

    mod current {
        use super::*;

        #[test]
        fn prints_current_vault() {
            let cmd = Obz::from_command("vaults current");
            let temp_path = cmd.temp_dir.to_path_buf();

            cmd.assert_stdout(format!(
                "Current vault is `main` at path `{}/main-vault/`\n",
                temp_path.display()
            ));
        }
    }
}
