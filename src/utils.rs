extern crate expanduser;

use expanduser::expanduser;
use std::io::Result;
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::path::PathBuf;


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
pub fn link_file(src: &str, dst: &str) -> Result<()> {
    let _dst = prepare_path(dst)?;

    if cfg!(target_family = "unix") {
        symlink(fs::canonicalize(src)?, _dst)?;
    } else {
        fs::hard_link(src, _dst)?;
    }

    Ok(())
}

/// Create the parent directories of a path and return the path with tildes
/// expanded.
fn prepare_path(path: &str) -> Result<PathBuf> {
    let _dst = expanduser(path)?;
    if let Some(_path) = _dst.parent() {
        fs::create_dir_all(_path)?;
    }
    if fs::symlink_metadata(&_dst).is_ok() {
        // Check for existing files, including broken symlinks
        fs::remove_file(&_dst)?;
    }
    Ok(_dst)
}
