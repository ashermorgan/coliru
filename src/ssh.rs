//! Remote dotfile installation utilities
//!
//! To send files to a remote machine via SCP, first stage them using
//! [`stage_file`], then transfer them using [`send_staged_files`].
//!
//! ```
//! let staging_dir = Path::new("/tmp/staging");
//! let host = "user@hostname";
//! stage_file("foo.sh", "~/foo.sh", staging_dir);
//! send_staged_files(staging_dir, host);
//! send_command("bash ~/foo.sh", host);
//! ```

use anyhow::{bail, anyhow, Context, Result};
use std::env;
use shellexpand::tilde_with_context;
use std::fs::{read_dir, remove_dir_all};
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};
use std::process::{Command, Stdio};
use super::local::copy_file;

/// Makes a relative path absolute according to a certain base directory
///
/// Paths begining with tildes are interpreted as absolute paths.
///
/// ```
/// assert_eq!(resolve_path("dir1/foo", "~/dir2"), "~/dir2/dir1/foo");
/// assert_eq!(resolve_path("/dir1/foo", "~/dir2"), "/dir1/foo");
/// assert_eq!(resolve_path("~/dir1/foo", "~/dir2"), "~/dir1/foo");
/// ```
pub fn resolve_path(src: &str, dir: &str) -> String {
    if !src.starts_with("~") && Path::new(src).is_relative() {
        return format!("{dir}/{src}")
    }
    src.to_owned()
}

/// Copies a file to an SCP staging directory
///
/// Tildes are expanded and relative paths are interpreted relative to the
/// remote user's home directory.
///
/// ```
/// // Prepare to transfer foo to ~/foo, bar to /bar, and baz to ~/baz
/// let staging_dir = Path::new("/tmp/staging");
/// stage_file("foo", "~/foo", staging_dir);
/// stage_file("bar", "/bar", staging_dir);
/// stage_file("baz", "baz", staging_dir);
/// ```
pub fn stage_file(src: &str, dst: &str, staging_dir: &Path) -> Result<()> {
    // Staging directories are used to copy multiple files at once while
    // automatically creating missing directories on the remote machine. The
    // example code above produces the following staging directory layout:
    //
    // /tmp/staging/
    // ├── home/
    // │   ├── baz
    // │   └── foo
    // └── root/
    //     └── bar

    let home_dir = staging_dir.join("home");
    let root_dir = staging_dir.join("root");
    let get_home_dir = || {
        Some::<String>(home_dir.to_string_lossy().into())
    };

    // Resolve ~/... paths to home staging directory:
    let mut _dst: PathBuf = (&tilde_with_context(dst, get_home_dir).to_mut())
                            .into();

    // Resolve relative paths to home staging directory:
    _dst = home_dir.join(_dst);

    // Resolve other absolute paths to root staging directory:
    if !_dst.starts_with(home_dir) {
        // Root should be / and C:\ on Unix and Windows respectively, but
        // iter().next() will return / and C:, so we must manually add another
        // path separator. (Duplicate slashes are ignored on Unix).
        let root = PathBuf::from(match _dst.iter().next() {
            Some(x) => Ok(x),
            None => Err(anyhow!("Failed to get root of {}", _dst.display())),
        }?).join(MAIN_SEPARATOR_STR);

        let dst_without_root = _dst.strip_prefix(root).with_context(|| {
            format!("Failed to strip root from {}", _dst.display())
        })?;
        _dst = root_dir.join(dst_without_root);
    }

    copy_file(src, _dst.to_string_lossy().to_mut())
}

/// Transfers the files in an SCP staging directory to a remote machine
///
/// `host` may be an SSH alias or a string in the form `user@hostname`. Use
/// [`stage_file`] to produce a staging directory. The contents of the staging
/// directory are deleted after they are successfully transferred.
///
/// ```
/// send_staged_files(Path::new("/tmp/staging"), "user@hostname");
/// ```
pub fn send_staged_files(staging_dir: &Path, host: &str) -> Result<()> {
    let home_dir = staging_dir.join("home");
    if home_dir.exists() {
        send_dir(home_dir.to_string_lossy().to_mut(), "~", host)?;
        remove_dir_all(&home_dir).with_context(|| {
            format!("Failed to remove staging dir {} after use",
                    &home_dir.display())
        })?;
    }
    let root_dir = staging_dir.join("root");
    if root_dir.exists() {
        send_dir(root_dir.to_string_lossy().to_mut(), "/", host)?;
        remove_dir_all(&root_dir).with_context(|| {
            format!("Failed to remove staging dir {} after use",
                    &root_dir.display())
        })?;
    }
    Ok(())
}

/// Copies a directory to another machine via SCP and merges it with a
/// destination directory
///
/// `host` may be an SSH alias or a string in the form `user@hostname`.
///
/// ```
/// send_dir("new_home", "~/", "user@hostname");
/// ```
fn send_dir(src: &str, dst: &str, host: &str) -> Result<()> {
    // To avoid the source directory being copied as a subdirectory of the
    // destination directory, we must send the contents of the directory
    // item by item.
    let items = read_dir(&src).with_context(|| {
        format!("Failed to list contents of {}", src)
    })?;
    for item in items {
        let _src = item.with_context(|| {
            format!("Failed to list contents of {}", src)
        })?.path();

        let mut cmd = Command::new("scp");
        cmd.stdout(Stdio::null());

        if env::var("COLIRU_TEST").is_ok() {
            cmd.args(["-o", "StrictHostKeyChecking=no", "-P", "2222"]);
        }
        cmd.args(["-r", &_src.to_string_lossy(), &format!("{host}:{dst}")]);

        let status = cmd.status().with_context(|| {
            format!("Failed to execute {:?}", cmd)
        })?;
        if !status.success() {
            bail!("SCP terminated unsuccessfully: {}", status);
        }
    }
    Ok(())
}

/// Executes a command on another machine via SSH
///
/// `host` may be an SSH alias or a string in the form `user@hostname`.
///
/// ```
/// send_command("echo 'Hello World'");
/// ```
pub fn send_command(command: &str, host: &str) -> Result<()> {
    let mut cmd = Command::new("ssh");
    if env::var("COLIRU_TEST").is_ok() {
        cmd.args(["-o", "StrictHostKeyChecking=no", "-p", "2222"]);
    }
    cmd.args([host, command]);

    let status = cmd.status().with_context(|| {
        format!("Failed to execute {:?}", cmd)
    })?;
    if !status.success() {
        bail!("SSH terminated unsuccessfully: {}", status);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;
    use crate::test_utils::{SSH_HOST, read_file, setup_integration, write_file};

    use regex::Regex;
    use std::fs;

    #[test]
    fn test_resolve_path_relative() {
        let result = resolve_path("dir1/foo", "~/dir2");

        assert_eq!(result, "~/dir2/dir1/foo");
    }

    #[test]
    fn test_resolve_path_tilde() {
        let result = resolve_path("~/dir1/foo", "~/dir2");

        assert_eq!(result, "~/dir1/foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_resolve_path_absolute() {
        let result = resolve_path("/dir1/foo", "~/dir2");

        assert_eq!(result, "/dir1/foo");
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn test_resolve_path_absolute() {
        let result = resolve_path("C:\\dir1\\foo", "~/dir2");

        assert_eq!(result, "C:\\dir1\\foo");
    }

    #[test]
    fn test_stage_file_tilde() {
        let tmp = setup_integration("test_stage_file_tilde");

        let src = tmp.local.join("foo");
        let dst = "~/dir/bar";
        let dst_real = tmp.local.join("home").join("dir").join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst, staging);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    fn test_stage_file_relative() {
        let tmp = setup_integration("test_stage_file_relative");

        let src = tmp.local.join("foo");
        let dst = "dir/bar";
        let dst_real = tmp.local.join("home").join("dir")
            .join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst, staging);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    fn test_stage_file_absolute() {
        let tmp = setup_integration("test_stage_file_absolute");

        let src = tmp.local.join("foo");
        let dst = "/dir/bar";
        let dst_real = tmp.local.join("root").join("dir").join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst, staging);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_staged_files_no_files() {
        let tmp = setup_integration("test_send_staged_files_no_files");

        let result = send_staged_files(&tmp.local, SSH_HOST);

        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_staged_files_home() {
        let tmp = setup_integration("test_send_staged_files_home");

        let src = tmp.local.join("home").join("test_send_staged_files_home");
        let src_foo = src.join("foo");
        let src_bar = src.join("dir").join("bar");
        fs::create_dir_all(&src_bar.parent().unwrap()).unwrap();
        write_file(&src_foo, "contents of foo");
        write_file(&src_bar, "contents of bar");

        let result = send_staged_files(&tmp.local, SSH_HOST);

        let dst_foo = tmp.ssh.join("foo");
        let dst_bar = tmp.ssh.join("dir").join("bar");
        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "contents of bar");
        assert_eq!(tmp.local.join("home").exists(), false);
        assert_eq!(tmp.local.join("root").exists(), false);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_staged_files_root() {
        let tmp = setup_integration("test_send_staged_files_root");

        let src = tmp.local.join("root").join("home").join("test")
            .join("test_send_staged_files_root");
        let src_foo = src.join("foo");
        let src_bar = src.join("dir").join("bar");
        fs::create_dir_all(&src_bar.parent().unwrap()).unwrap();
        write_file(&src_foo, "contents of foo");
        write_file(&src_bar, "contents of bar");

        let result = send_staged_files(&tmp.local, SSH_HOST);

        let dst_foo = tmp.ssh.join("foo");
        let dst_bar = tmp.ssh.join("dir").join("bar");
        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "contents of bar");
        assert_eq!(tmp.local.join("home").exists(), false);
        assert_eq!(tmp.local.join("root").exists(), false);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_dir_basic() {
        let tmp = setup_integration("test_send_dir_basic");

        write_file(&tmp.local.join("foo"), "contents of foo");
        write_file(&tmp.local.join("bar"), "contents of bar");

        let dst = "~/test_send_dir_basic";
        let dst_foo = tmp.ssh.join("foo");
        let dst_bar = tmp.ssh.join("bar");

        let result = send_dir(tmp.local.to_str().unwrap(), dst, SSH_HOST);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "contents of bar");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_dir_nested_dir() {
        let tmp = setup_integration("test_send_dir_nested_dir");

        let src_foo = tmp.local.join("foo");
        let src_bar = tmp.local.join("dir").join("bar");
        write_file(&src_foo, "contents of foo");
        fs::create_dir_all(&src_bar.parent().unwrap()).unwrap();
        write_file(&src_bar, "contents of bar");

        let dst = "~/test_send_dir_nested_dir";
        let dst_foo = tmp.ssh.join("foo");
        let dst_bar = tmp.ssh.join("dir").join("bar");

        let result = send_dir(tmp.local.to_str().unwrap(), dst, SSH_HOST);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "contents of bar");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_dir_merge_dir() {
        let tmp = setup_integration("test_send_dir_merge_dir");

        let src_bar = tmp.local.join("dir").join("bar");
        fs::create_dir_all(src_bar.parent().unwrap()).unwrap();
        write_file(&src_bar, "new contents of bar");

        let dst = "~/test_send_dir_merge_dir";
        let dst_foo = tmp.ssh.join("foo");
        let dst_bar = tmp.ssh.join("dir").join("bar");
        let dst_baz = tmp.ssh.join("dir").join("baz");
        write_file(&dst_foo, "old contents of foo");
        fs::create_dir_all(&dst_bar.parent().unwrap()).unwrap();
        write_file(&dst_bar, "old contents of bar");
        write_file(&dst_baz, "old contents of baz");

        let result = send_dir(tmp.local.to_str().unwrap(), dst, SSH_HOST);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "old contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "new contents of bar");
        assert_eq!(dst_baz.exists(), true);
        assert_eq!(read_file(&dst_baz), "old contents of baz");
    }

    #[test]
    fn test_send_dir_bad_host() {
        let tmp = setup_integration("test_send_dir_bad_host");

        write_file(&tmp.local.join("foo"), "contents of foo");
        write_file(&tmp.local.join("bar"), "contents of bar");

        let dst = "~/test_send_dir_bad_host";
        let bad_host = "fake@coliru.test.internal"; // Will be a DNS error

        let result = send_dir(tmp.local.to_str().unwrap(), dst, bad_host);
        let expected = Regex::new("SCP terminated unsuccessfully: \
                                   exit (status|code): \\d+").unwrap();

        assert_eq!(result.is_ok(), false);
        assert_eq!(expected.is_match(&result.unwrap_err().to_string()), true);
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_command_basic() {
        let tmp = setup_integration("test_send_command_basic");

        let dst = "~/test_send_command_basic/foo";
        let dst_real = tmp.ssh.join("foo");
        let cmd = format!("echo 'contents of foo' > {}", dst);

        let result = send_command(&cmd, SSH_HOST);

        assert_eq!(result.is_ok(), true);
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo\n");
    }

    #[test]
    fn test_send_command_bad_host() {
        let _tmp = setup_integration("test_send_command_bad_host");

        let cmd = format!("echo Hello World");
        let bad_host = "fake@coliru.test.internal"; // Will be a DNS error

        let result = send_command(&cmd, bad_host);
        let expected = Regex::new("SSH terminated unsuccessfully: \
                                   exit (status|code): \\d+").unwrap();

        assert_eq!(result.is_ok(), false);
        assert_eq!(expected.is_match(&result.unwrap_err().to_string()), true);
    }
}
