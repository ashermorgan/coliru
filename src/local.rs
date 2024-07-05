//! Local dotfile installation utilities
//!
//! ```
//! copy_file("foo", "~/foo");
//! link_file("bar", "~/bar");
//! run_command("echo 'Hello world'");
//! ```

use shellexpand::tilde;
use std::io;
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::path::{PathBuf, absolute};
use std::process::Command;

/// Copies the contents of a file to another file
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary.
///
/// ```
/// copy_file("foo", "~/foo");
/// ```
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    if absolute(src)? == absolute(dst)? { return Ok(()); }
    let _dst = prepare_path(dst)?;
    fs::copy(src, _dst)?;
    Ok(())
}

/// Creates a symbolic link to a file
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary. On non-Unix platforms, a hard link will be created instead.
///
/// ```
/// link_file("bar", "~/bar");
/// ```
#[cfg(target_family = "unix")]
pub fn link_file(src: &str, dst: &str) -> io::Result<()> {
    if absolute(src)? == absolute(dst)? { return Ok(()); }
    let _dst = prepare_path(dst)?;
    symlink(fs::canonicalize(src)?, _dst)?;
    Ok(())
}
#[cfg(not(target_family = "unix"))]
pub fn link_file(src: &str, dst: &str) -> io::Result<()> {
    if absolute(src)? == absolute(dst)? { return Ok(()); }
    let _dst = prepare_path(dst)?;
    fs::hard_link(src, _dst)?;
    Ok(())
}

/// Creates the parent directories of a path, deletes the file if it exists, and
/// returns the path with tildes expanded
///
/// ```
/// prepare_path("~/foo");
/// ```
fn prepare_path(path: &str) -> io::Result<PathBuf> {
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

/// Executes a command using `sh` on Unix and `cmd` on Windows
///
/// ```
/// run_command("echo 'Hello world'");
/// ```
pub fn run_command(command: &str) -> Result<(), String>
{
    let status;
    if cfg!(target_family = "unix") {
        status = Command::new("sh")
            .args(["-c", command])
            .status()
            .map_err(|why| why.to_string())?;
    } else {
        status = Command::new("cmd.exe")
            .args(["/C", command])
            .status()
            .map_err(|why| why.to_string())?;
    }
    if status.success() {
        Ok(())
    } else {
        Err(format!("Process exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_integration, write_file};

    #[test]
    fn test_copy_file_create_dirs() {
        let tmp = setup_integration("test_copy_file_create_dirs");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("dir1").join("dir2").join("bar");
        write_file(src, "old contents of foo");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    fn test_copy_file_same_file() {
        let tmp = setup_integration("test_copy_file_same_file");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("foo");
        write_file(src, "contents of foo");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "contents of foo");
    }

    #[test]
    fn test_copy_file_existing_file() {
        let tmp = setup_integration("test_copy_file_existing_file");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("bar");
        write_file(src, "old contents of foo");
        write_file(dst, "old contents of bar");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_copy_file_existing_broken_symlink() {
        let tmp = setup_integration("test_copy_file_existing_broken_symlink");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("bar");
        write_file(src, "old contents of foo");
        symlink("missing", dst).unwrap();

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_copy_file_tilde_expansion() {
        let tmp = setup_integration("test_copy_file_tilde_expansion");

        let src = &tmp.local.join("foo");
        let dst = &tmp.home.join("dir").join("bar");
        let dst_tilde = "~/test_copy_file_tilde_expansion/dir/bar";
        write_file(src, "old contents of foo");

        let result = copy_file(src.to_str().unwrap(), dst_tilde);

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    fn test_link_file_create_dirs() {
        let tmp = setup_integration("test_link_file_create_dirs");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("dir1").join("dir2").join("bar");
        write_file(src, "old contents of foo");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    fn test_link_file_same_file() {
        let tmp = setup_integration("test_link_file_same_file");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("foo");
        write_file(src, "contents of foo");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "contents of foo");
    }

    #[test]
    fn test_link_file_existing_file() {
        let tmp = setup_integration("test_link_file_existing_file");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("bar");
        write_file(src, "old contents of foo");
        write_file(dst, "old contents of bar");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_existing_broken_symlink() {
        let tmp = setup_integration("test_link_file_existing_broken_symlink");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("bar");
        write_file(src, "old contents of foo");
        symlink("missing", dst).unwrap();

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_tilde_expansion() {
        let tmp = setup_integration("test_link_file_tilde_expansion");

        let src = &tmp.local.join("foo");
        let dst = &tmp.home.join("dir").join("bar");
        let dst_tilde = "~/test_link_file_tilde_expansion/dir/bar";
        write_file(src, "old contents of foo");

        let result = link_file(src.to_str().unwrap(), dst_tilde);

        write_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_relative_source() {
        let dir = PathBuf::from("tests/.temp/ssh/test_link_file_relative_source");
        fs::create_dir_all(&dir).unwrap();

        let src = absolute(&dir.join("foo")).unwrap();
        let src_rel = "tests/.temp/ssh/test_link_file_relative_source/foo";
        let dst = &dir.join("dir1").join("dir2").join("bar");
        write_file(&src, "old contents of foo");

        let result = link_file(src_rel, dst.to_str().unwrap());

        write_file(&src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        let link = fs::read_link(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
        assert_eq!(link, src); // src changed to absolute path

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_command_successful() {
        let tmp = setup_integration("test_run_command_successful");

        let src = &tmp.local.join("foo");
        write_file(src, "exit 0");

        let result = run_command(&format!("sh {}", src.to_str().unwrap()));

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_command_successful() {
        let tmp = setup_integration("test_run_command_successful");

        let src = &tmp.local.join("foo.bat");
        write_file(src, "exit 0");

        let result = run_command(src.to_str().unwrap());

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_command_failure() {
        let tmp = setup_integration("test_run_command_failure");

        let src = &tmp.local.join("foo");
        write_file(src, "exit 2");

        let result = run_command(&format!("sh {}", src.to_str().unwrap()));

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.unwrap_err(), "Process exited with exit status: 2");
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_command_failure() {
        let tmp = setup_integration("test_run_command_failure");

        let src = &tmp.local.join("foo.bat");
        write_file(src, "exit 1");

        let result = run_command(src.to_str().unwrap());

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.unwrap_err(), "Process exited with exit code: 1");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_command_arguments() {
        let tmp = setup_integration("test_run_command_arguments");

        let src = &tmp.local.join("foo");
        let dst = &tmp.local.join("bar");
        write_file(src, &format!("echo $@ > {}", dst.to_str().unwrap()));

        let result = run_command(&format!("sh {} arg1 arg2",
                                          src.to_str().unwrap()));

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "arg1 arg2\n");
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_command_arguments() {
        let tmp = setup_integration("test_run_command_arguments");

        let src = &tmp.local.join("foo.bat");
        let dst = &tmp.local.join("bar");
        write_file(src, &format!("echo %* > {}", dst.to_str().unwrap()));

        let result = run_command(&format!("{} arg1 arg2",
                                          src.to_str().unwrap()));

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "arg1 arg2 \r\n");
    }
}
