use shellexpand::tilde_with_context;
use std::fs::{read_dir, remove_dir_all};
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};
use std::process::Command;
use super::local::copy_file;

/// Copy a file to an SCP staging directory
///
/// The destination directory structure will be recreated in the staging
/// directory under either the home or root subdirectories. This staging system
/// allows missing directories to be created automatically on the remote
/// machine.
pub fn stage_file(src: &str, dst: &str, staging_dir: &Path) -> Result<(), String> {
    let home_dir = staging_dir.join("home");
    let root_dir = staging_dir.join("root");
    let get_home_dir = || {
        Some::<String>(home_dir.to_string_lossy().into())
    };

    // Resolve ~/... paths to home staging directory:
    let mut _dst: PathBuf = (&tilde_with_context(dst, get_home_dir).to_mut())
                            .into();

    // Resolve relative paths to home staging directory:
    _dst = home_dir.join(".coliru").join(_dst);

    // Resolve other absolute paths to root staging directory:
    if !_dst.starts_with(home_dir) {
        // Root should be / and C:\ on Unix and Windows respectively, but
        // iter().next() will return / and C:, so we must manually add another
        // path separator. (Duplicate slashes are ignored on Unix).
        let root = PathBuf::from(match _dst.iter().next() {
            Some(x) => Ok(x),
            None => Err(String::from("Destination path does not have root")),
        }?).join(MAIN_SEPARATOR_STR);
        _dst = root_dir.join(_dst.strip_prefix(root)
                             .map_err(|why| why.to_string())?);
    }

    copy_file(src, _dst.to_string_lossy().to_mut())
        .map_err(|why| why.to_string())
}

/// Transfer the files in an SCP staging directory to a remote machine
pub fn send_staged_files(staging_dir: &Path, host: &str) -> Result<(), String> {
    let home_dir = staging_dir.join("home");
    if home_dir.exists() {
        send_dir(home_dir.to_string_lossy().to_mut(), "~", host)?;
        remove_dir_all(home_dir).map_err(|why| why.to_string())?;
    }
    let root_dir = staging_dir.join("root");
    if root_dir.exists() {
        send_dir(root_dir.to_string_lossy().to_mut(), "/", host)?;
        remove_dir_all(root_dir).map_err(|why| why.to_string())?;
    }
    Ok(())
}

/// Copy a directory to another machine via SCP and merge it with a destination
/// directory
fn send_dir(src: &str, dst: &str, host: &str) -> Result<(), String> {
    // To avoid the source directory being copied as a subdirectory of the
    // destination directory, we must send the contents of the directory
    // item by item.
    for item in read_dir(&src).map_err(|why| why.to_string())? {
        let _src = item.map_err(|why| why.to_string())?.path();

        let mut cmd = Command::new("scp");
        if host == "test@localhost" {
            // SSH options and port for test server hard coded for now
            cmd.args(["-o", "StrictHostKeyChecking=no", "-P", "2222"]);
        }
        cmd.args(["-r", &_src.to_string_lossy(), &format!("{host}:{dst}")]);

        let status = cmd.status().map_err(|why| why.to_string())?;
        if !status.success() {
            return Err(format!("SCP exited with {status}"));
        }
    }
    Ok(())
}

/// Execute a command on another machine via SSH
#[allow(dead_code)]
pub fn send_command(command: &str, host: &str) -> Result<(), String> {
    let mut cmd = Command::new("ssh");
    if host == "test@localhost" {
        // SSH options and port for test server hard coded for now
        cmd.args(["-o", "StrictHostKeyChecking=no", "-p", "2222"]);
    }
    cmd.args([host, command]);

    let status = cmd.status().map_err(|why| why.to_string())?;
    if !status.success() {
        return Err(format!("SSH exited with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;
    use crate::common::{SSH_HOST, read_file, setup_integration, write_file};

    use std::fs;

    #[test]
    fn test_stage_file_tilde() {
        let tmp = setup_integration("test_stage_file_tilde");

        let src = tmp.local.join("foo");
        let dst = "~/dir/bar";
        let dst_real = tmp.local.join("home").join("dir").join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst, staging);

        assert_eq!(result, Ok(()));
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    fn test_stage_file_relative() {
        let tmp = setup_integration("test_stage_file_relative");

        let src = tmp.local.join("foo");
        let dst = "dir/bar";
        let dst_real = tmp.local.join("home").join(".coliru").join("dir")
            .join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst, staging);

        assert_eq!(result, Ok(()));
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

        assert_eq!(result, Ok(()));
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_staged_files_no_files() {
        let tmp = setup_integration("test_send_staged_files_no_files");

        let result = send_staged_files(&tmp.local, SSH_HOST);

        assert_eq!(result, Ok(()));
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
        assert_eq!(result, Ok(()));
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
        assert_eq!(result, Ok(()));
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

        assert_eq!(result, Ok(()));
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

        assert_eq!(result, Ok(()));
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

        assert_eq!(result, Ok(()));
        assert_eq!(dst_foo.exists(), true);
        assert_eq!(read_file(&dst_foo), "old contents of foo");
        assert_eq!(dst_bar.exists(), true);
        assert_eq!(read_file(&dst_bar), "new contents of bar");
        assert_eq!(dst_baz.exists(), true);
        assert_eq!(read_file(&dst_baz), "old contents of baz");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_command_basic() {
        let tmp = setup_integration("test_send_command_basic");

        let dst = "~/test_send_command_basic/foo";
        let dst_real = tmp.ssh.join("foo");
        let cmd = format!("echo 'contents of foo' > {}", dst);

        let result = send_command(&cmd, SSH_HOST);

        assert_eq!(result, Ok(()));
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo\n");
    }
}
