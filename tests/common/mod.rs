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
pub struct TempDir {
    /// A temporary directory that is located at or in $HOME on Unix
    pub home: PathBuf,

    /// A temporary directory that is located at or under the CWD
    pub local: PathBuf,

    /// A temporary directory that is mounted to the SSH server under $HOME
    pub ssh: PathBuf,
}
impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.ssh).unwrap();
        fs::remove_dir_all(&self.local).unwrap();
        fs::remove_dir_all(&self.home).unwrap();
    }
}
impl TempDir {
    fn new(name: &str) -> TempDir {
        // The CWD of the current process is always the repository root
        let dir = env::current_dir().unwrap().join("tests").join(".temp");

        let home = dir.join("home").join(name);
        let local = dir.join("local").join(name);
        let ssh = dir.join("ssh").join(name);

        assert_eq!(home.exists(), false);
        assert_eq!(local.exists(), false);
        assert_eq!(ssh.exists(), false);

        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&local).unwrap();
        fs::create_dir_all(&ssh).unwrap();

        TempDir { home, local, ssh }
    }
}

/// Initializes temporary directories for integration tests
///
/// On Unix, $HOME is set to the parent directory of the home temporary
/// directory, which is the same for all integration tests. This prevents issues
/// when tests are run in multiple threads.
pub fn setup_integration(name: &str) -> TempDir {
    let dirs = TempDir::new(name);
    if cfg!(target_family = "unix") {
        env::set_var("HOME", dirs.home.parent().unwrap());
    }
    dirs
}

/// Initializes temporary directories and a coliru Command for e2e tests
///
/// The Command's CWD is set to the local temporary directory, and on Unix, the
/// Command's $HOME variable is set to the home temporary directory.
pub fn setup_e2e(name: &str) -> (TempDir, Command) {
    let dirs = TempDir::new(name);

    let exe = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join(format!("../coliru{}", env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(exe);
    cmd.current_dir(&dirs.local);
    if cfg!(target_family = "unix") {
        cmd.env("HOME", &dirs.home);
    }

    (dirs, cmd)
}

/// Initializes temporary directories and a coliru Command with the --host
/// argument set for e2e tests
pub fn setup_e2e_ssh(name: &str) -> (TempDir, Command) {
    let (dirs, mut cmd) = setup_e2e(name);
    cmd.args(["--host", SSH_HOST]);
    (dirs, cmd)
}

/// Create a basic manifest file and its associated dotfiles in a directory
///
/// All occurances of the string "~/" in examples/manifest.yml will be replaced
/// with the value of home_dir.
pub fn copy_manifest(dir: &Path, home_dir: &str) {
    // Copy files from examples
    let examples = env::current_exe().unwrap().parent().unwrap().to_path_buf()
        .join("../../../examples");
    let copy_file = |name: &str| {
        fs::copy(examples.join(name), &dir.join(name)).unwrap();
    };
    copy_file("script.bat");
    copy_file("script.sh");

    // Create manifest file with "~/" replaced for home_dir
    let mut manifest = read_file(&examples.join("manifest.yml"));
    manifest = manifest.replace("~/", home_dir);
    write_file(&dir.join("manifest.yml"), &manifest);

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
