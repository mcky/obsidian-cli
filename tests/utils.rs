use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug)]
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

        Obz { cmd, temp_dir }.with_editor("")
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

    pub fn assert_content<P>(mut self, file_path: P, file_content: &str) -> Self
    where
        P: AsRef<Path>,
    {
        self.cmd.assert().success();
        self.temp_dir
            .child(file_path)
            .assert(predicate::path::exists())
            .assert(predicates::str::contains(file_content.trim()));

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
            .stdout(predicates::str::contains(stdout_match));
        self
    }

    pub fn assert_stderr<S>(mut self, stderr_match: S) -> Self
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .failure()
            .stderr(predicates::str::contains(stderr_match));
        self
    }
}

/// Create a TempDir and clone our example vault into it
pub fn create_fixtures() -> TempDir {
    let dir = TempDir::new().expect("failed to create new TempDir");

    dir.copy_from("tests/fixtures", &["*.md"]).unwrap();

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
