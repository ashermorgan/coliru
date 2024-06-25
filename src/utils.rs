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
    let _dst = prepare_path(dst)?;
    symlink(fs::canonicalize(src)?, _dst)?;
    Ok(())
}
#[cfg(not(target_family = "unix"))]
pub fn link_file(src: &str, dst: &str) -> io::Result<()> {
    let _dst = prepare_path(dst)?;
    fs::hard_link(src, _dst)?;
    Ok(())
}

/// Create the parent directories of a path and return the path with tildes
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

/// Execute a local shell script, optionally with a command prefix or postfix.
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
mod tests {
    use super::*;

    use std::env;
    use std::io::Write;
    use std::path::Path;

    // Adapted from ripgrep tests (crates/ignore/src/lib.rs)
    struct TempDir(PathBuf);
    impl Drop for TempDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    impl TempDir {
        // Will cause a panic if name has already been used
        fn create(name: &str) -> TempDir {
            let path = env::temp_dir().join("coliru-tests").join(name);
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }
    }

    fn create_file(path: &Path, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    #[test]
    fn test_copy_file_create_dirs() {
        let tmp = TempDir::create("test_copy_file_create_dirs");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("dir1").join("dir2").join("bar");
        create_file(src, "old contents of foo");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    fn test_copy_file_existing_file() {
        let tmp = TempDir::create("test_copy_file_existing_file");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("bar");
        create_file(src, "old contents of foo");
        create_file(dst, "old contents of bar");

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_copy_file_existing_broken_symlink() {
        let tmp = TempDir::create("test_copy_file_existing_broken_symlink");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("bar");
        create_file(src, "old contents of foo");
        symlink("missing", dst).unwrap();

        let result = copy_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_copy_file_tilde_expansion() {
        let tmp = TempDir::create("test_copy_file_tilde_expansion");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("dir1").join("dir2").join("bar");
        create_file(src, "old contents of foo");
        env::set_var("HOME", tmp.0.join("dir1").to_str().unwrap());

        let result = copy_file(src.to_str().unwrap(), "~/dir2/bar");

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "old contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_create_dirs() {
        let tmp = TempDir::create("test_link_file_create_dirs");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("dir1").join("dir2").join("bar");
        create_file(src, "old contents of foo");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_existing_file() {
        let tmp = TempDir::create("test_link_file_existing_file");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("bar");
        create_file(src, "old contents of foo");
        create_file(dst, "old contents of bar");

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_existing_broken_symlink() {
        let tmp = TempDir::create("test_link_file_existing_broken_symlink");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("bar");
        create_file(src, "old contents of foo");
        symlink("missing", dst).unwrap();

        let result = link_file(src.to_str().unwrap(), dst.to_str().unwrap());

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_link_file_tilde_expansion() {
        let tmp = TempDir::create("test_link_file_tilde_expansion");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("dir1").join("dir2").join("bar");
        create_file(src, "old contents of foo");
        env::set_var("HOME", tmp.0.join("dir1").to_str().unwrap());

        let result = link_file(src.to_str().unwrap(), "~/dir2/bar");

        create_file(src, "new contents of foo");
        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "new contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_script_successful() {
        let tmp = TempDir::create("test_run_script_successful");

        let src = &tmp.0.join("foo");
        create_file(src, "exit 0");

        let result = run_script(src.to_str().unwrap(), "bash", "");

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_script_failure() {
        let tmp = TempDir::create("test_run_script_failure");

        let src = &tmp.0.join("foo");
        create_file(src, "exit 2");

        let result = run_script(src.to_str().unwrap(), "bash", "");

        assert_eq!(result.is_ok(), false);
        assert_eq!(result.unwrap_err(), "Process exited with exit status: 2");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_run_script_postfix() {
        let tmp = TempDir::create("test_run_script_postfix");

        let src = &tmp.0.join("foo");
        let dst = &tmp.0.join("bar");
        create_file(src, &format!("echo $@ > {}", dst.to_str().unwrap()));

        let result = run_script(src.to_str().unwrap(), "bash", "arg1 arg2");

        let contents = fs::read_to_string(dst).unwrap();
        assert_eq!(result.is_ok(), true);
        assert_eq!(contents, "arg1 arg2\n");
    }
}
