use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// An isolated machine for the CLI: its own LITHEMUX_HOME, HOME and Claude
/// config directory, a minimal PATH and a fake login shell, so the real
/// `claude` of the machine can never be found or run.
pub struct Sandbox {
    pub dir: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        for folder in ["home", "user", "claude-config", "work"] {
            fs::create_dir_all(dir.path().join(folder)).unwrap();
        }
        Sandbox { dir }
    }

    pub fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    pub fn work(&self) -> PathBuf {
        self.dir.path().join("work").canonicalize().unwrap()
    }

    pub fn claude_config(&self) -> PathBuf {
        self.dir.path().join("claude-config")
    }

    pub fn std_command(&self, args: &[&str]) -> std::process::Command {
        let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin("lithemux"));
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("SHELL", fake("fake-shell"))
            .env("HOME", self.dir.path().join("user"))
            .env("CLAUDE_CONFIG_DIR", self.claude_config())
            .env("LITHEMUX_HOME", self.home())
            .args(args);
        command
    }

    pub fn lithemux(&self, args: &[&str]) -> Command {
        Command::from_std(self.std_command(args))
    }

    pub fn run(&self, args: &[&str]) -> String {
        let output = self.lithemux(args).output().unwrap();
        assert!(
            output.status.success(),
            "lithemux {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    pub fn fail(&self, args: &[&str]) -> String {
        let output = self.lithemux(args).output().unwrap();
        assert!(!output.status.success(), "lithemux {args:?} should fail");
        String::from_utf8(output.stderr).unwrap()
    }

    /// The demo category of the plan's acceptance test.
    pub fn with_demo(self) -> Self {
        let work = self.work();
        self.run(&[
            "cat",
            "add",
            "demo",
            "--name",
            "Demo",
            "--root",
            work.to_str().unwrap(),
        ]);
        self.run(&[
            "new",
            "demo/planificadora",
            "--role",
            "Planifica y revisa",
            "--mother",
        ]);
        self.run(&["new", "demo/ejecutora", "--role", "Implementa las fases"]);
        self.run(&["new", "demo/documental", "--role", "Requerimientos"]);
        self
    }

    pub fn use_fake_claude(&self) {
        self.run(&["config", "set", "claude_path", &fake_claude()]);
    }

    pub fn fake_claude_log(&self) -> Vec<String> {
        fs::read_to_string(self.home().join("fake-claude.log"))
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

pub fn fake_claude() -> String {
    fake("fake-claude")
}

fn fake(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fakes")
        .join(name)
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
}
