//! Coliru testing utilities
//!
//! These utilities create and manage resources used in integration and
//! end-to-end tests, including temporary directories, processes running coliru
//! commands, and test dotfile repositories. There are also functions for basic
//! I/O and capturing command output. All temporary directories are located
//! under the `.temp` directory and are unique to each test according to name.

#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The SSH test server
pub const SSH_HOST: &str = "test@localhost"; // TODO: add explicit port

/// A set of temporary directories that are automatically deleted when the value
/// is dropped
pub struct TempDirs {
    /// A temporary directory that is located at or in `~/` on Unix
    pub home: PathBuf,

    /// A temporary directory that is located at or under the current working
    /// directory
    pub local: PathBuf,

    /// A temporary directory that is mounted to the SSH server under `~/`
    pub ssh: PathBuf,

    /// A temporary directory that is mounted to the SSH server under
    /// `~/.coliru`
    pub ssh_cwd: PathBuf,
}
impl Drop for TempDirs {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.home).unwrap();
        fs::remove_dir_all(&self.local).unwrap();
        fs::remove_dir_all(&self.ssh).unwrap();
        fs::remove_dir_all(&self.ssh_cwd).unwrap();
    }
}
impl TempDirs {
    /// Creates a new set of temporary directories with a certain name
    ///
    /// ```
    /// let dirs = TempDirs::new("test_foo");
    /// ```
    fn new(name: &str) -> TempDirs {
        // The working directory of the main process is always the repository
        // root
        let dir = env::current_dir().unwrap().join("tests").join(".temp");

        let home = dir.join("home").join(name);
        let local = dir.join("local").join(name);
        let ssh = dir.join("ssh").join(name);
        let ssh_cwd = dir.join("ssh").join(".coliru").join(name);

        assert_eq!(home.exists(), false);
        assert_eq!(local.exists(), false);
        assert_eq!(ssh.exists(), false);
        assert_eq!(ssh_cwd.exists(), false);

        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&local).unwrap();
        fs::create_dir_all(&ssh).unwrap();
        fs::create_dir_all(&ssh_cwd).unwrap();

        TempDirs { home, local, ssh, ssh_cwd }
    }
}

/// Initializes temporary directories for integration tests
///
/// On Unix, `$HOME` is set to the parent directory of the home temporary
/// directory, which is the same for all integration tests. This prevents issues
/// when tests are run in multiple threads.
///
/// ```
/// let dirs = setup_integration("test_foo");
/// ```
pub fn setup_integration(name: &str) -> TempDirs {
    let dirs = TempDirs::new(name);
    if cfg!(target_family = "unix") {
        env::set_var("HOME", dirs.home.parent().unwrap());
    }
    dirs
}

/// Initializes temporary directories and a coliru Command for E2E tests
///
/// The Command's working directory is set to the local temporary directory, and
/// on Unix, the Command's `$HOME` variable is set to the home temporary
/// directory.
///
/// ```
/// let (dirs, cmd) = setup_e2e("test_foo");
/// ```
fn setup_e2e(name: &str) -> (TempDirs, Command) {
    let dirs = TempDirs::new(name);

    let exe = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join(format!("../coliru{}", env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(exe);
    cmd.current_dir(&dirs.local);
    if cfg!(target_family = "unix") {
        cmd.env("HOME", &dirs.home);
    }

    (dirs, cmd)
}

/// Initializes temporary directories and a coliru Command for local E2E tests
///
/// A test dotfile repository is copied to the working directory (mapped to the
/// `local` temporary directory), to be installed to the home directory on Unix
/// (mapped to the `home` temporary directory) and the current working directory
/// on Windows (mapped to the `local` temporary directory).
///
/// ```
/// let (dirs, cmd) = setup_e2e_local("test_foo");
/// ```
pub fn setup_e2e_local(name: &str) -> (TempDirs, Command) {
    let (dirs, cmd) = setup_e2e(name);

    // It's difficult to mock $HOME on Windows, so install dotfiles in CWD
    let home_dir = if cfg!(target_family = "unix") { "~/" } else { "" };
    copy_manifest(&dirs.local, home_dir, "");

    (dirs, cmd)
}

/// Initializes temporary directories and a coliru Command for SSH E2E tests
///
/// A test dotfile repository is copied to the working directory (mapped to the
/// `local` temporary directory), to be installed over SSH to `~/test_name/`
/// (mapped to the `ssh` temporary directory), with scripts copied to
/// `~/.coliru/test_name/` (mapped to the `ssh_cwd` temporary directory). The
/// `--host` flag is set to the test SSH server.
///
/// ```
/// let (dirs, cmd) = setup_e2e_ssh("test_foo");
/// ```
pub fn setup_e2e_ssh(name: &str) -> (TempDirs, Command) {
    let (dirs, mut cmd) = setup_e2e(name);
    cmd.args(["--host", SSH_HOST]);

    // Replace ~/ and scripts/ with custom directory to isolate SSH tests
    copy_manifest(&dirs.local, &format!("~/{name}/"), &format!("{name}/"));

    (dirs, cmd)
}

/// Initializes a basic dotfiles repository in a directory
///
/// The dotfiles from `examples/test/` are used as a starting template. All
/// occurrences of `~/` and `scripts/` are replaced with `home_dir` and
/// `script_dir`, respectively, to allow dotfiles to be isolated across tests
/// when necessary.
///
/// ```
/// copy_manifest(&Path::new("/tmp/dotfiles/"), "~/", "scripts/");
/// ```
fn copy_manifest(dir: &Path, home_dir: &str, script_dir: &str) {
    let examples = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join("../../../examples/test");

    let copy_file = |path: &str| {
        let mut contents = read_file(&examples.join(path));
        contents = contents.replace("~/", home_dir);
        contents = contents.replace("scripts/", script_dir);
        let dst = path.replace("scripts/", script_dir);
        write_file(&dir.join(dst), &contents);
    };

    copy_file("manifest.yml");
    fs::create_dir_all(&dir.join(script_dir)).unwrap();
    copy_file("scripts/script.bat");
    copy_file("scripts/script.sh");
    copy_file("bashrc");
    copy_file("gitconfig");
    copy_file("vimrc");

}

/// Writes a string to a file, overwriting it if it already exists
pub fn write_file(path: &Path, contents: &str) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

/// Reads the contents of a file into a string
pub fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

/// Run a command and return its output (stdout and stderr) and exit status
pub fn run_command(cmd: &mut Command) -> (String, String, Option<i32>) {
    let output = cmd.output().unwrap();
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}
