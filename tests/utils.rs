use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

pub struct Obz {
    pub cmd: assert_cmd::Command,
    pub temp_dir: TempDir,
}

impl Obz {
    pub fn from_command(command_str: &str) -> Self {
        let temp_dir = create_fixtures();

        let mut cmd = Command::cargo_bin("obz").expect("failed to construct obz command");

        cmd.current_dir(&temp_dir);

        let args = command_str.split(" ");
        for arg in args {
            cmd.arg(arg);
        }

        let config_path = temp_dir.child("./cfg/obz.yml");

        let initial_cfg_file = format!(
            indoc! {
            r#"current_vault: main
                vaults:
                - name: main
                  path: {dir}/main-vault/
                - name: secondary
                  path: {dir}/another/path
                "#},
            dir = temp_dir.display()
        );

        cmd.env("OBZ_CONFIG", config_path.display().to_string());

        Obz { cmd, temp_dir }
            .with_editor("")
            .with_config_file(&*&initial_cfg_file)
    }

    pub fn with_config_file(self, cfg_file: &str) -> Self {
        self.temp_dir
            .child("./cfg/obz.yml")
            .write_str(&cfg_file)
            .expect("should be able to write to config file");

        self
    }

    pub fn with_editor<S>(mut self, editor_script: S) -> Self
    where
        S: Into<String>,
    {
        let mock_editor = create_editor_script(editor_script, &self.temp_dir);
        self.cmd
            .env("EDITOR", &mock_editor.path().to_str().unwrap());

        self
    }

    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.cmd.env(key, val);
        self
    }

    pub fn assert_created<P>(mut self, file_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.cmd.assert().success();
        self.temp_dir
            .child(file_path)
            .assert(predicate::path::exists());

        self
    }

    pub fn assert_content<P, S>(mut self, file_path: P, file_content: S) -> Self
    where
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.cmd.assert().success();
        self.temp_dir
            .child(file_path)
            .assert(predicate::path::exists())
            .assert(predicate::str::diff(file_content.into()));

        self
    }

    pub fn assert_success(mut self) -> Self {
        self.cmd.assert().success();
        self
    }

    pub fn assert_stdout<S>(mut self, stdout_match: S) -> Self
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .success()
            .stdout(predicate::str::diff(stdout_match.into()));
        self
    }

    pub fn assert_stdout_contains<S>(mut self, stdout_match: S) -> Self
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .success()
            .stdout(predicate::str::contains(stdout_match.into()));
        self
    }

    pub fn assert_stderr<S>(mut self, stderr_match: S) -> Self
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .failure()
            .stderr(predicate::str::diff(stderr_match.into()));
        self
    }
}

/// Create a TempDir and clone our example vault into it
pub fn create_fixtures() -> TempDir {
    let dir = TempDir::new().expect("failed to create new TempDir");

    let main_vault = dir.child("./main-vault");
    main_vault.copy_from("tests/fixtures", &["*.md"]).unwrap();

    let secondary_vault = dir.child("./another/path");
    secondary_vault
        .child("from-another-vault.md")
        .write_str("This note is from the secondary vault").unwrap();

    dir
}

pub fn exec_with_fixtures(s: &str) -> (TempDir, Command) {
    let dir = create_fixtures();
    let cmd = get_cmd(&dir, s);

    (dir, cmd)
}

/// Allows constructing a command as an entire string, e.g. `get_cmd(&dir, "notes create foo")`
/// instead of using the `.arg` builder pattern
pub fn get_cmd(dir: &TempDir, command_str: &str) -> Command {
    let mut cmd = Command::cargo_bin("obz").expect("failed to construct obz command");

    cmd.current_dir(&dir);

    let args = command_str.split(" ");
    for arg in args {
        cmd.arg(arg);
    }

    return cmd;
}

pub fn create_editor_script<S>(content: S, temp_dir: &TempDir) -> ChildPath
where
    S: Into<String>,
{
    // Create a mock editor script, we can verify it's been called
    // because it will append to the file
    let mock_editor = temp_dir.child("mock_editor.sh");
    let editor_content: String = format!("#!/bin/sh\n{}", content.into());

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

    mock_editor
}
