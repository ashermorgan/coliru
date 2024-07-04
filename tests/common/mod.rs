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
    /// A temporary directory that is located at or in $HOME on Unix
    pub home: PathBuf,

    /// A temporary directory that is located at or under the CWD
    pub local: PathBuf,

    /// A temporary directory that is mounted to the SSH server under $HOME
    pub ssh: PathBuf,
}
impl Drop for TempDirs {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.ssh).unwrap();
        fs::remove_dir_all(&self.local).unwrap();
        fs::remove_dir_all(&self.home).unwrap();
    }
}
impl TempDirs {
    fn new(name: &str) -> TempDirs {
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

        TempDirs { home, local, ssh }
    }
}

/// Initializes temporary directories for integration tests
///
/// On Unix, $HOME is set to the parent directory of the home temporary
/// directory, which is the same for all integration tests. This prevents issues
/// when tests are run in multiple threads.
pub fn setup_integration(name: &str) -> TempDirs {
    let dirs = TempDirs::new(name);
    if cfg!(target_family = "unix") {
        env::set_var("HOME", dirs.home.parent().unwrap());
    }
    dirs
}

/// Initializes temporary directories and a coliru Command for e2e tests
///
/// The Command's CWD is set to the local temporary directory, and on Unix, the
/// Command's $HOME variable is set to the home temporary directory.
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

/// Initializes temporary directories and a coliru Command for local e2e tests
///
/// A test dotfile repo is copied to the local folder, to be installed to $HOME
/// on Unix and CWD on Windows.
pub fn setup_e2e_local(name: &str) -> (TempDirs, Command) {
    let (dirs, cmd) = setup_e2e(name);

    // It's difficult to mock $HOME on Windows, so install dotfiles in CWD
    let home_dir = if cfg!(target_family = "unix") { "~/" } else { "" };
    copy_manifest(&dirs.local, home_dir, "");

    (dirs, cmd)
}

/// Initializes temporary directories and a coliru Command for ssh e2e tests
///
/// A test dotfile repo is copied to the local folder, to be installed to
/// ~/test_name/ with scripts copied to ~/.coliru/test_name/, and the --host
/// argument is set to the test SSH server.
pub fn setup_e2e_ssh(name: &str) -> (TempDirs, Command) {
    let (dirs, mut cmd) = setup_e2e(name);
    cmd.args(["--host", SSH_HOST]);

    // Replace ~/ and scripts/ with custom directory to isolate SSH tests
    copy_manifest(&dirs.local, &format!("~/{name}/"), &format!("{name}/"));

    (dirs, cmd)
}

/// Create a basic manifest file and its associated dotfiles in a directory
///
/// All occurrences of the string "~/" and "scripts/" (e.g. in manifest.yml and
/// scripts/*) will be replaced with the value of home_dir and script_dir
/// respectively to ensures that dotfiles are isolated across tests when
/// necessary.
pub fn copy_manifest(dir: &Path, home_dir: &str, script_dir: &str) {
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
