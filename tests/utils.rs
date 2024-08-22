use assert_cmd::prelude::*;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indoc::indoc;
use predicates::prelude::*;
use rexpect::session::PtyReplSession;
use std::fs;
use std::path::Path;

pub struct Obx {
    pub cmd: std::process::Command,
    pub temp_dir: TempDir,
}

impl Obx {
    pub fn from_command(command_str: &str) -> Self {
        let temp_dir = create_fixtures();

        let mut cmd =
            std::process::Command::cargo_bin("obx").expect("failed to construct obx command");

        cmd.current_dir(&temp_dir);

        let args = command_str.split(" ");
        for arg in args {
            cmd.arg(arg);
        }

        let config_path = temp_dir.child("./config/obx/");

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

        cmd.env("OBX_CONFIG_DIR", config_path.display().to_string());

        Obx { cmd, temp_dir }
            .with_editor("")
            .with_config_file(&*&initial_cfg_file)
    }

    pub fn spawn_interactive(self, timeout: Option<u64>) -> anyhow::Result<PtyReplSession> {
        // Take our usual cmd but instead of asserting on it, convert it into
        // a string, then split it into the `cd $dir` and `cmd $args` parts
        let cmd_str = &format!("{:?}", self.cmd);

        let cmd_parts = cmd_str.split(" && ").collect::<Vec<&str>>();
        let [cd_cmd, bin_cmd] = &cmd_parts[..] else {
            panic!("couldn't split cmd_parts")
        };

        let mut p = rexpect::spawn_bash(timeout)?;

        p.send_line(&cd_cmd).unwrap();
        p.send_line(&bin_cmd).unwrap();

        Ok(p)
    }

    pub fn with_config_file(self, cfg_file: &str) -> Self {
        self.temp_dir
            .child("./config/obx/config.yml")
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

    dir.copy_from("tests/fixtures", &["*.md"]).unwrap();

    dir
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
