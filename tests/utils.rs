use std::ffi;
use std::path::Path;

use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;

#[derive(Debug)]
pub struct ObzTest {
    cmd: assert_cmd::Command,
    pub temp_dir: TempDir,
}

impl ObzTest {
    pub fn from_command(command_str: &str) -> Self {
        let temp_dir = create_fixtures();

        let mut cmd = Command::cargo_bin("obz").expect("failed to construct obz command");

        cmd.current_dir(&temp_dir);

        let args = command_str.split(" ");
        for arg in args {
            cmd.arg(arg);
        }

        ObzTest { cmd, temp_dir }
    }

    pub fn assert_created<P>(mut self, file_path: P) -> ()
    where
        P: AsRef<Path>,
    {
        self.cmd.assert().success();
        self.temp_dir
            .child(file_path)
            .assert(predicate::path::exists());
    }

    pub fn assert_content<P>(mut self, file_path: P, file_content: &str) -> ()
    where
        P: AsRef<Path>,
    {
        self.cmd.assert().success();
        self.temp_dir
            .child(file_path)
            .assert(predicate::path::exists())
            .assert(predicates::str::contains(file_content.trim()));

        ()
    }

    pub fn assert_success(mut self) -> () {
        self.cmd.assert().success();
    }

    pub fn assert_stdout<S>(mut self, stdout_match: S) -> ()
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .success()
            .stdout(predicates::str::contains(stdout_match));
    }

    pub fn assert_stderr<S>(mut self, stderr_match: S) -> ()
    where
        S: Into<String>,
    {
        self.cmd
            .assert()
            .failure()
            .stderr(predicates::str::contains(stderr_match));
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