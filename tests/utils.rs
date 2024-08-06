use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;

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

// Custom assertions

#[macro_export]
macro_rules! assert_success {
    ($command:expr) => {{
        let (_dir, mut cmd) = exec_with_fixtures($command);
        cmd.assert().success();
    }};
    ($command:expr, $expected:expr) => {{
        let (_dir, mut cmd) = exec_with_fixtures($command);

        cmd.assert()
            .success()
            .stdout(predicates::str::contains($expected.trim()));
    }};
}

#[macro_export]
macro_rules! assert_stderr {
    ($command:expr, $expected:expr) => {{
        let (_dir, mut cmd) = exec_with_fixtures($command);
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains($expected.trim()));
    }};
}

#[macro_export]
macro_rules! assert_created {
    ($command:expr, $file_path:expr) => {{
        let (dir, mut cmd) = exec_with_fixtures($command);
        cmd.assert().success();
        dir.child($file_path).assert(predicate::path::exists());
    }};
    ($command:expr, $file_path:expr, $content:expr) => {{
        let (dir, mut cmd) = exec_with_fixtures($command);
        cmd.assert().success();
        dir.child($file_path)
            .assert(predicate::path::exists())
            .assert(predicates::str::contains($content.trim()));
    }};
}
