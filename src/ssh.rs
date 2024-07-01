use std::process::Command;

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
#[cfg(target_family = "unix")]
mod tests {
    use super::*;
    use crate::common::{SSH_HOST, read_file, setup_integration, write_file};

    use std::fs;

    #[test]
    fn test_send_dir_basic() {
        let tmp = setup_integration("test_send_dir_basic");

        write_file(&tmp.local.join("foo"), "contents of foo");
        write_file(&tmp.local.join("bar"), "contents of bar");

        let result = send_dir(&tmp.local.to_str().unwrap(), "~/", SSH_HOST);

        assert_eq!(result.is_ok(), true);
        let contents1 = read_file(&tmp.ssh.join("foo"));
        assert_eq!(contents1, "contents of foo");
        let contents2 = read_file(&tmp.ssh.join("bar"));
        assert_eq!(contents2, "contents of bar");
    }

    #[test]
    fn test_send_dir_nested() {
        let tmp = setup_integration("test_send_dir_nested");

        write_file(&tmp.local.join("foo"), "contents of foo");
        fs::create_dir_all(&tmp.local.join("dir")).unwrap();
        write_file(&tmp.local.join("dir").join("bar"), "contents of bar");

        let result = send_dir(tmp.local.to_str().unwrap(), "~/", SSH_HOST);

        assert_eq!(result.is_ok(), true);
        let contents1 = read_file(&tmp.ssh.join("foo"));
        assert_eq!(contents1, "contents of foo");
        let contents2 = read_file(&tmp.ssh.join("dir").join("bar"));
        assert_eq!(contents2, "contents of bar");
    }
}
