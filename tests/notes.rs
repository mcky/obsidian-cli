use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn note_view_accepts_full_paths() {
    let dir = create_fixtures();
    get_cmd(&dir)
        .arg("view")
        .arg("some-note.md")
        .assert()
        .success();
}

#[test]
fn note_view_accepts_partial_paths() {
    let dir = create_fixtures();
    get_cmd(&dir)
        .arg("view")
        .arg("some-note")
        .assert()
        .success();
}

#[test]
fn note_view_fails_for_missing_files() {
    let dir = create_fixtures();
    get_cmd(&dir)
        .arg("view")
        .arg("does-not-exist.md")
        .assert()
        .failure()
        .stderr("Could not read note does-not-exist.md");
    // .stderr(predicate::str::contains("xxx not read file"));
}

#[test]
fn note_view_prints_note() {
    let dir = create_fixtures();
    get_cmd(&dir)
        .arg("view")
        .arg("some-note.md")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "The contents of the file named some-note.md",
        ));
}

fn create_fixtures() -> TempDir {
    let dir = TempDir::new().expect("failed to create new TempDir");

    dir.child("some-note.md")
        .write_str("The contents of the file named some-note.md")
        .expect("failed to create file in TempDir");

    dir
}

fn get_cmd(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("obz").expect("failed to construct obz command");

    cmd.current_dir(&dir).arg("notes");

    return cmd;
}
