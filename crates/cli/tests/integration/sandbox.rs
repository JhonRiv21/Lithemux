use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use assert_cmd::Command;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tempfile::TempDir;

/// An isolated machine for the CLI: its own LITHEMUX_HOME, HOME and Claude
/// config directory, a minimal PATH and a fake login shell, so the real
/// `claude` of the machine can never be found or run.
pub struct Sandbox {
    pub dir: TempDir,
    fake_shell: String,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        for folder in ["home", "user", "claude-config", "work"] {
            fs::create_dir_all(dir.path().join(folder)).unwrap();
        }
        Sandbox {
            dir,
            fake_shell: fake("fake-shell"),
        }
    }

    /// Uses another of the fake login shells in `core/tests/fakes`.
    pub fn with_fake_shell(mut self, name: &str) -> Self {
        self.fake_shell = fake(name);
        self
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

    /// Everything the CLI may see: nothing of the machine gets through.
    fn environment(&self) -> [(&'static str, String); 5] {
        let path = |path: PathBuf| path.display().to_string();
        [
            ("PATH", "/usr/bin:/bin".to_string()),
            ("SHELL", self.fake_shell.clone()),
            ("HOME", path(self.dir.path().join("user"))),
            ("CLAUDE_CONFIG_DIR", path(self.claude_config())),
            ("LITHEMUX_HOME", path(self.home())),
        ]
    }

    pub fn std_command(&self, args: &[&str]) -> std::process::Command {
        let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin("lithemux"));
        command.env_clear().envs(self.environment()).args(args);
        command
    }

    /// Starts the CLI on a real terminal, as in any terminal tab.
    pub fn start_on_a_terminal(&self, args: &[&str]) -> OnATerminal {
        let pair = native_pty_system().openpty(PtySize::default()).unwrap();
        let mut command = CommandBuilder::new(assert_cmd::cargo::cargo_bin("lithemux"));
        command.env_clear();
        for (name, value) in self.environment() {
            command.env(name, value);
        }
        command.args(args);
        let child = pair.slave.spawn_command(command).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let printed = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = reader.read_to_string(&mut text);
            text
        });
        OnATerminal {
            keyboard: pair.master.take_writer().unwrap(),
            _terminal: pair.master,
            child,
            printed,
        }
    }

    /// Runs the CLI on a real terminal and returns what it printed there.
    pub fn run_on_a_terminal(&self, args: &[&str]) -> String {
        self.start_on_a_terminal(args).finish().1
    }

    /// Whether a `claude` found through the PATH of a fake shell can be
    /// launched. The launcher never runs a `claude` that sits in a temporary
    /// folder (docs/decisiones.md D-05), and the fakes do when the repository
    /// itself was checked out in one.
    pub fn fakes_on_the_path_are_launchable() -> bool {
        let fakes = fake("on-path");
        ![
            "/tmp/",
            "/private/tmp/",
            "/var/folders/",
            "/private/var/folders/",
        ]
        .iter()
        .any(|temporary| fakes.starts_with(temporary))
    }

    /// Waits until the fake claude wrote the file that says it is running.
    pub fn wait_for_fake_claude(&self) {
        let started = self.home().join("fake-claude.pid");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !started.exists() {
            assert!(Instant::now() < deadline, "the fake claude never started");
            std::thread::sleep(Duration::from_millis(10));
        }
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

    /// A number the fake claude left in one of its files, such as its pid.
    pub fn fake_claude_number(&self, file: &str) -> i32 {
        fs::read_to_string(self.home().join(file))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }

    /// Starts `open` and returns once the fake claude is running.
    pub fn open_in_background(&self, target: &str) -> std::process::Child {
        let child = self
            .std_command(&["open", target])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        self.wait_for_fake_claude();
        child
    }

    pub fn fake_claude_log(&self) -> Vec<String> {
        fs::read_to_string(self.home().join("fake-claude.log"))
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

/// The CLI running on a pseudo terminal owned by the test.
pub struct OnATerminal {
    keyboard: Box<dyn Write + Send>,
    _terminal: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    printed: std::thread::JoinHandle<String>,
}

impl OnATerminal {
    /// What the user types, control characters included.
    pub fn type_keys(&mut self, keys: &[u8]) {
        self.keyboard.write_all(keys).unwrap();
        self.keyboard.flush().unwrap();
    }

    /// The exit code and everything that was printed on the terminal.
    pub fn finish(mut self) -> (u32, String) {
        let code = self.child.wait().unwrap().exit_code();
        drop(self.keyboard);
        drop(self._terminal);
        (code, self.printed.join().unwrap())
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
