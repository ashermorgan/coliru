use shellexpand::tilde_with_context;
use std::path::{MAIN_SEPARATOR_STR, PathBuf};
use std::process::Command;
use super::local::copy_file;

/// Copy a file to an SCP staging directory
///
/// The destination directory structure will be recreated in the staging
/// directory under either the home or root subdirectories. This staging system
/// allows for many files to transferred at once and for missing directories to
/// be created automatically on the remote machine.
#[allow(dead_code)]
fn stage_file(src: &str, dst: &str, staging_dir: &str) -> Result<(), String> {
    let _staging_dir = PathBuf::from(staging_dir);
    let home_dir = _staging_dir.join("home");
    let root_dir = _staging_dir.join("root");
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
            None => Err(String::from("Destination path does not have root")),
        }?).join(MAIN_SEPARATOR_STR);
        _dst = root_dir.join(_dst.strip_prefix(root)
                             .map_err(|why| why.to_string())?);
    }

    copy_file(src, _dst.to_string_lossy().to_mut())
        .map_err(|why| why.to_string())
}

/// Recursively copy a directory to another machine via SCP
#[allow(dead_code)]
fn send_dir(src: &str, dst: &str, host: &str) -> Result<(), String> {
    let mut cmd = Command::new("scp");
    if cfg!(test) {
        // SSH options and port for test server hard coded for now
        cmd.args(["-o", "StrictHostKeyChecking=no", "-P", "2222"]);
    }
    cmd.args(["-r", src, &format!("{host}:{dst}")]);

    let status = cmd.status().map_err(|why| why.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("SCP exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]

    use super::*;
    use crate::common::{SSH_HOST, read_file, setup_integration, write_file};

    use std::fs;

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_dir_basic() {
        let tmp = setup_integration("test_send_dir_basic");

        write_file(&tmp.local.join("foo"), "contents of foo");
        write_file(&tmp.local.join("bar"), "contents of bar");

        let result = send_dir(&tmp.local.to_str().unwrap(), "~/", SSH_HOST);

        assert_eq!(result, Ok(()));
        let contents1 = read_file(&tmp.ssh.join("foo"));
        assert_eq!(contents1, "contents of foo");
        let contents2 = read_file(&tmp.ssh.join("bar"));
        assert_eq!(contents2, "contents of bar");
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn test_send_dir_nested() {
        let tmp = setup_integration("test_send_dir_nested");

        write_file(&tmp.local.join("foo"), "contents of foo");
        fs::create_dir_all(&tmp.local.join("dir")).unwrap();
        write_file(&tmp.local.join("dir").join("bar"), "contents of bar");

        let result = send_dir(tmp.local.to_str().unwrap(), "~/", SSH_HOST);

        assert_eq!(result, Ok(()));
        let contents1 = read_file(&tmp.ssh.join("foo"));
        assert_eq!(contents1, "contents of foo");
        let contents2 = read_file(&tmp.ssh.join("dir").join("bar"));
        assert_eq!(contents2, "contents of bar");
    }

    #[test]
    fn test_stage_file_tilde() {
        let tmp = setup_integration("test_stage_file_tilde");

        let src = tmp.local.join("foo");
        let dst = "~/dir/bar";
        let dst_real = tmp.local.join("home").join("dir").join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst,
                                staging.to_str().unwrap());

        assert_eq!(result, Ok(()));
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }

    #[test]
    fn test_stage_file_relative() {
        let tmp = setup_integration("test_stage_file_relative");

        let src = tmp.local.join("foo");
        let dst = "dir/bar";
        let dst_real = tmp.local.join("home").join("dir").join("bar");
        let staging  = &tmp.local;
        write_file(&src, "contents of foo");

        let result = stage_file(src.to_str().unwrap(), dst,
                                staging.to_str().unwrap());

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

        let result = stage_file(src.to_str().unwrap(), dst,
                                staging.to_str().unwrap());

        assert_eq!(result, Ok(()));
        assert_eq!(dst_real.exists(), true);
        assert_eq!(read_file(&dst_real), "contents of foo");
    }
}
