#![allow(unused_imports)]

//! End to end tests that test installation behavior on a remote machine via SSH

mod test_utils;

use test_utils::*;
use std::fs::remove_file;

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_standard() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);

    let expected = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_standard/.gitconfig
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_standard/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_standard/.vimrc
[2/3] Copy test_ssh_standard/script.sh to {SSH_HOST}:~/.coliru/test_ssh_standard/script.sh
[2/3] Run sh test_ssh_standard/script.sh arg1 linux on {SSH_HOST}
script.sh called with arg1 linux
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc"));
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    let log_contents = read_file(&dirs.ssh_cwd.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_alternate_tag_rules_1() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "linux", "^windows"]);

    let expected = format!("\
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_1/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_1/.vimrc
[2/3] Copy test_ssh_run_alternate_tag_rules_1/script.sh to {SSH_HOST}:~/.coliru/test_ssh_run_alternate_tag_rules_1/script.sh
[2/3] Run sh test_ssh_run_alternate_tag_rules_1/script.sh arg1 linux ^windows on {SSH_HOST}
script.sh called with arg1 linux ^windows
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_exists = dirs.ssh.join(".gitconfig").exists();
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc"));
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    let log_contents = read_file(&dirs.ssh_cwd.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_exists, false);
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux ^windows\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_alternate_tag_rules_2() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "macos"]);

    let expected = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.gitconfig
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.vimrc
[2/3] Copy test_ssh_run_alternate_tag_rules_2/script.sh to {SSH_HOST}:~/.coliru/test_ssh_run_alternate_tag_rules_2/script.sh
[2/3] Run sh test_ssh_run_alternate_tag_rules_2/script.sh arg1 macos on {SSH_HOST}
script.sh called with arg1 macos
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc"));
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    let log_contents = read_file(&dirs.ssh_cwd.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
fn test_ssh_dry_run() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);

    let expected = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_dry_run/.gitconfig (DRY RUN)
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_dry_run/.bashrc (DRY RUN)
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_dry_run/.vimrc (DRY RUN)
[2/3] Copy test_ssh_dry_run/script.sh to {SSH_HOST}:~/.coliru/test_ssh_dry_run/script.sh (DRY RUN)
[2/3] Run sh test_ssh_dry_run/script.sh arg1 linux on {SSH_HOST} (DRY RUN)
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/run
    let bash_exists = dirs.ssh.join(".bashrc").exists();
    let git_exists = dirs.ssh.join(".gitconfig").exists();
    let vim1_exists = dirs.ssh.join(".vimrc").exists();
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    let log_exists = dirs.ssh_cwd.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_copy() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);

    let expected = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_copy/.gitconfig
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_copy/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_copy/.vimrc
[2/3] Copy test_ssh_copy/script.sh to {SSH_HOST}:~/.coliru/test_ssh_copy/script.sh
[2/3] Run sh test_ssh_copy/script.sh arg1 linux on {SSH_HOST}
script.sh called with arg1 linux
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc"));
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    let log_contents = read_file(&dirs.ssh_cwd.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_failure() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    write_file(&dirs.local.join("test_ssh_run_failure/script.sh"), "exit 1");

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_run_failure/.gitconfig
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_failure/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_failure/.vimrc
[2/3] Copy test_ssh_run_failure/script.sh to {SSH_HOST}:~/.coliru/test_ssh_run_failure/script.sh
[2/3] Run sh test_ssh_run_failure/script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: SSH terminated unsuccessfully: exit status: 1\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, &expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc"));
    let vim2_exists = dirs.ssh.join("_vimrc").exists();
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_missing_file() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    remove_file(&dirs.local.join("vimrc")).unwrap();

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_missing_file/.gitconfig
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_missing_file/.bashrc
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_missing_file/.vimrc
[2/3] Copy test_ssh_missing_file/script.sh to {SSH_HOST}:~/.coliru/test_ssh_missing_file/script.sh
[2/3] Run sh test_ssh_missing_file/script.sh arg1 linux on {SSH_HOST}
script.sh called with arg1 linux
");
    let expected_stderr = "  Error: Failed to copy vimrc to staging directory: \
                           No such file or directory (os error 2)\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, &expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig"));
    let log_contents = read_file(&dirs.ssh_cwd.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}
