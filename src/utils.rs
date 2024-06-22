use shellexpand::tilde;
use std::io::Result;
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::path::{PathBuf, absolute};
use std::process::Command;


/// Copies the contents of a local file to another local file.
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary.
pub fn copy_file(src: &str, dst: &str) -> Result<()> {
    let _dst = prepare_path(dst)?;
    fs::copy(src, _dst)?;
    Ok(())
}

/// Creates a symbolic link to a local file.
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary. On non-Unix platforms, a hard link will be created instead.
#[cfg(target_family = "unix")]
pub fn link_file(src: &str, dst: &str) -> Result<()> {
    let _dst = prepare_path(dst)?;
    symlink(fs::canonicalize(src)?, _dst)?;
    Ok(())
}
#[cfg(not(target_family = "unix"))]
pub fn link_file(src: &str, dst: &str) -> Result<()> {
    let _dst = prepare_path(dst)?;
    fs::hard_link(src, _dst)?;
    Ok(())
}

/// Create the parent directories of a path and return the path with tildes
/// expanded.
fn prepare_path(path: &str) -> Result<PathBuf> {
    let _dst: PathBuf = (&tilde(path).to_mut()).into();
    if let Some(_path) = _dst.parent() {
        fs::create_dir_all(_path)?;
    }
    if fs::symlink_metadata(&_dst).is_ok() {
        // Check for existing files, including broken symlinks
        fs::remove_file(&_dst)?;
    }
    Ok(_dst)
}

/// Execute a local shell script.
///
/// Uses sh on Unix and PowerShell on Windows.
pub fn run_script(path: &str) -> Result<()> {
    if cfg!(target_family = "unix") {
        Command::new("sh")
            .arg("-c")
            .arg(fs::canonicalize(path)?)
            .status()?;
    } else {
        // Use absolute() instead of canonicalize() to avoid incompatible paths:
        // https://github.com/rust-lang/rust/issues/42869
        Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command"])
            .arg(absolute(path)?)
            .status()?;
    }
    Ok(())
}
