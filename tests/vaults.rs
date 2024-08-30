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
            let cmd = Obx::from_command("vaults create path/to/new-vault");
            let _ = &cmd.temp_dir.child("path/to/new-vault/file.md").touch();

            cmd.assert_stdout("Created vault new-vault\n");
        }

        #[test]
        fn accepts_name_param() {
            let cmd = Obx::from_command("vaults create path/to/new-vault --name another-vault");
            let _ = &cmd.temp_dir.child("path/to/new-vault/file.md").touch();

            cmd.assert_stdout("Created vault another-vault\n");
        }

        #[test]
        fn fails_on_missing_dir() {
            Obx::from_command("vaults create /does/not/exist").assert_stderr(
                "Could not create vault at path `/does/not/exist`, directory not found\n",
            );
        }

        #[test]
        fn fails_if_not_dir() {
            Obx::from_command("vaults create ./main-vault/simple-note.md").assert_stderr(
                "Could not create vault at path `./main-vault/simple-note.md`, path must be a directory\n",
            );
        }

        #[test]
        fn persists_changes() {
            let create_cmd = Obx::from_command("vaults create path/to/new/vault");
            let _ = create_cmd
                .temp_dir
                .child("path/to/new/vault/file.md")
                .touch();

            let expected_vault_path = create_cmd.temp_dir.child("path/to/new/vault");

            // Ensure list_cmd is reading from the same temp_dir as create_cmd, so we
            // pick up the persisted changes
            let mut list_cmd = Obx::from_command("vaults list");
            let tmp_config_path = create_cmd.temp_dir.child("./config/obx/");
            list_cmd.env("OBX_CONFIG_DIR", tmp_config_path.display().to_string());

            // Keep a reference to the Obx instance so we don't drop the tmp dir
            let _ = &create_cmd.assert_success();

            let expected_path = format!("{}", expected_vault_path.display());

            list_cmd.assert_stdout_contains(expected_path);
        }
    }

    mod list {
        use super::*;

        #[test]
        fn prints_vault_list() {
            let cmd = Obx::from_command("vaults list").with_config_file(indoc! {
                r"
                current_vault: some-vault
                vaults:
                    - name: some-vault
                      path: /some/path
            "
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
            let cmd = Obx::from_command("vaults list -f json").with_config_file(indoc! {
                r"
                current_vault: some-vault
                vaults:
                    - name: some-vault
                      path: /some/path
            "
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
        fn accepts_vault_name() {
            Obx::from_command("vaults switch secondary")
                .assert_stdout("Switched to vault secondary\n");
        }

        #[test]
        #[ignore = "TODO: test interactive mode"]
        fn prompts_when_name_not_provided() {
            Obx::from_command("vaults switch");
        }

        #[test]
        fn fails_on_missing_vault() {
            Obx::from_command("vaults switch does-not-exist")
                .assert_stderr("Could not switch to vault `does-not-exist`, vault doesn't exist\n");
        }

        #[test]
        fn persists_changes() {
            let switch_cmd = Obx::from_command("vaults switch secondary");

            // Ensure curr_cmd is reading from the same temp_dir as switch_cmd, so we
            // pick up the persisted changes
            let mut curr_cmd = Obx::from_command("vaults current");
            let tmp_config_path = switch_cmd.temp_dir.child("./config/obx/");
            curr_cmd.env("OBX_CONFIG_DIR", tmp_config_path.display().to_string());

            // Keep a reference to the Obx instance so we don't drop the tmp dir
            let _x = switch_cmd.assert_success();

            curr_cmd.assert_stdout_contains("secondary");
        }
    }

    mod current {
        use super::*;

        #[test]
        fn prints_current_vault() {
            let cmd = Obx::from_command("vaults current");
            let temp_path = cmd.temp_dir.to_path_buf();

            cmd.assert_stdout(format!(
                "Current vault is `main` at path `{}/main-vault/`\n",
                temp_path.display()
            ));
        }
    }
}
