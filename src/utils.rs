use shellexpand::tilde;
use std::io;
use std::fs;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::path::{PathBuf, absolute};
use std::process::Command;

/// Copies the contents of a local file to another local file.
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary.
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    if absolute(src)? == absolute(dst)? { return Ok(()); }
    let _dst = prepare_path(dst)?;
    fs::copy(src, _dst)?;
    Ok(())
}

/// Creates a symbolic link to a local file.
///
/// Tildes are expanded if present and the destination file is overwritten if
/// necessary. On non-Unix platforms, a hard link will be created instead.
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

/// Creates the parent directories of a path and return the path with tildes
/// expanded.
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

/// Executes a local shell script, optionally with a command prefix or postfix.
///
/// Uses sh on Unix and PowerShell on Windows.
pub fn run_script(path: &str, prefix: &str, postfix: &str) -> Result<(), String>
{
    // Use absolute() to avoid incompatible "UNC" paths on Windows:
    // https://github.com/rust-lang/rust/issues/42869
    let _path = absolute(path).map_err(|why| why.to_string())?;
    let status;
    if cfg!(target_family = "unix") {
        status = Command::new("sh")
            .arg("-c")
            .arg(format!("{} {} {}", prefix, _path.display(), postfix))
            .status()
            .map_err(|why| why.to_string())?;
    } else {
        status = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command"])
            .arg(format!("{} {} {}", prefix, _path.display(), postfix))
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
#[path = "../tests/common/mod.rs"]
mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use common::{setup_integration, write_file};

    #[test]
    fn test_copy_file_create_dirs() {
        let tmp = setup_integration("test_copy_file_create_dirs");

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("dir1").join("dir2").join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("foo");
        write_file(src, "contents of foo");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "contents of foo");
    }

    #[test]
    fn test_copy_file_existing_file() {
        let tmp = setup_integration("test_copy_file_existing_file");

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("dir").join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("dir1").join("dir2").join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("foo");
        write_file(src, "contents of foo");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "contents of foo");
    }

    #[test]
    fn test_link_file_existing_file() {
        let tmp = setup_integration("test_link_file_existing_file");

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("bar");
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

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("dir").join("bar");
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
    fn test_run_script_successful() {
        let tmp = setup_integration("test_run_script_successful");

        let src = &tmp.dir.join("foo");
        write_file(src, "exit 0");

        let result = run_script(src.to_str().unwrap(), "sh", "");

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_script_successful() {
        let tmp = setup_integration("test_run_script_successful");

        let src = &tmp.dir.join("foo.bat");
        write_file(src, "exit 0");

        let result = run_script(src.to_str().unwrap(), "", "");

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_script_failure() {
        let tmp = setup_integration("test_run_script_failure");

        let src = &tmp.dir.join("foo");
        write_file(src, "exit 2");

        let result = run_script(src.to_str().unwrap(), "sh", "");

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.unwrap_err(), "Process exited with exit status: 2");
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_script_failure() {
        let tmp = setup_integration("test_run_script_failure");

        let src = &tmp.dir.join("foo.bat");
        write_file(src, "exit 1");

        let result = run_script(src.to_str().unwrap(), "", "");

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.unwrap_err(), "Process exited with exit code: 1");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_script_postfix() {
        let tmp = setup_integration("test_run_script_postfix");

        let src = &tmp.dir.join("foo");
        let dst = &tmp.dir.join("bar");
        write_file(src, &format!("echo $@ > {}", dst.to_str().unwrap()));

        let result = run_script(src.to_str().unwrap(), "sh", "arg1 arg2");

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "arg1 arg2\n");
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_run_script_postfix() {
        let tmp = setup_integration("test_run_script_postfix");

        let src = &tmp.dir.join("foo.bat");
        let dst = &tmp.dir.join("bar");
        write_file(src, &format!("echo %* > {}", dst.to_str().unwrap()));

        let result = run_script(src.to_str().unwrap(), "", "arg1 arg2");

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "arg1 arg2 \r\n");
    }
}
