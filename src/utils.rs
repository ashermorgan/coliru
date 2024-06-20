extern crate expanduser;

use expanduser::expanduser;
use std::io::Result;
use std::fs::{copy, create_dir_all};

/// Copies the contents of a local file to another local file.
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary.
pub fn copy_file(src: &str, dst: &str) -> Result<()> {
    let _dst = expanduser(dst)?;
    if let Some(path) = _dst.parent() {
        create_dir_all(path)?;
    }
    copy(src, _dst)?;
    Ok(())
}
