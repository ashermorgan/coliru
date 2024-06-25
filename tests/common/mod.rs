use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Stores the path to a temporary directory that is automatically deleted
/// when the value is dropped.
///
/// Adapted from ripgrep's tests (crates/ignore/src/lib.rs)
pub struct TempDir {
    dir: PathBuf
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

/// Creates a temporary directory with a certain name and create a new coliru
/// Command with $HOME and the CWD set the the temporary directory.
///
/// Adapted from ripgrep's tests (tests/utils.rs)
pub fn setup(name: &str) -> (TempDir, Command) {
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

/// Returns the stdout of a command as a String.
pub fn get_output(cmd: &mut Command) -> String {
    String::from_utf8_lossy(&cmd.output().unwrap().stdout).into_owned()
}
