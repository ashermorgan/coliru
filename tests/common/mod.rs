#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The SSH test server
pub const SSH_HOST: &str = "test@localhost"; // TODO: add explicit port

/// Stores the path to a temporary directory that is automatically deleted
/// when the value is dropped.
///
/// Adapted from ripgrep's tests (crates/ignore/src/lib.rs)
pub struct TempDir {
    pub dir: PathBuf
}
impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).unwrap();
    }
}
impl TempDir {
    fn new(name: &str) -> TempDir {
        let dir = env::temp_dir().join("coliru-tests").join(name);
        assert_eq!(dir.exists(), false);
        fs::create_dir_all(&dir).unwrap();
        TempDir { dir }
    }
}

/// Creates a temporary directory with a certain name and sets $HOME to the
/// parent directory.
///
/// All tests in this module use the same values for $HOME, which prevents
/// issues when tests are run in multiple threads.
pub fn setup_integration(name: &str) -> TempDir {
    let dir = TempDir::new(name);
    let root = dir.dir.parent().unwrap();
    if cfg!(target_family = "unix") {
        env::set_var("HOME", root);
    }
    dir
}

/// Creates a temporary directory with a certain name and create a new coliru
/// Command with $HOME and the CWD set the the temporary directory.
///
/// Adapted from ripgrep's tests (tests/utils.rs)
pub fn setup_e2e(name: &str) -> (TempDir, Command) {
    let dir = TempDir::new(name);

    let exe = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join(format!("../coliru{}", env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(exe);
    cmd.current_dir(&dir.dir);

    if cfg!(target_family = "unix") {
        cmd.env("HOME", &dir.dir);
    }

    (dir, cmd)
}

/// Prepares a temporary directory and a new Command for an e2e test, with the
/// --host argument already provided
pub fn setup_e2e_ssh(name: &str) -> (TempDir, Command) {
    let (dir, mut cmd) = setup_e2e(name);
    cmd.args(["--host", SSH_HOST]);
    (dir, cmd)
}

/// Create a basic manifest file and its associated dotfiles in a directory
pub fn copy_manifest(dir: &Path) {
    // Copy files from examples
    let examples = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join("../../../examples");
    let copy_file = |name: &str| {
        fs::copy(examples.join(name), &dir.join(name)).unwrap();
    };
    copy_file("script.bat");
    copy_file("script.sh");
    copy_file("manifest.yml");
    copy_file("manifest-windows-test.yml");

    // Create simplified config files
    write_file(&dir.join("bashrc"), "bash #1");
    write_file(&dir.join("gitconfig"), "git #1");
    write_file(&dir.join("vimrc"), "vim #1");

}

/// Writes a string to a file, overwriting it if it already exists.
pub fn write_file(path: &Path, contents: &str) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

/// Reads the contents of a file into a string.
pub fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

/// Returns the stdout of a command as a String.
pub fn stdout_to_string(cmd: &mut Command) -> String {
    String::from_utf8_lossy(&cmd.output().unwrap().stdout).into_owned()
}

/// Returns the stderr of a command as a String.
pub fn stderr_to_string(cmd: &mut Command) -> String {
    String::from_utf8_lossy(&cmd.output().unwrap().stderr).into_owned()
}
