/// End to end tests that test specific installation behavior on a remote
/// machine via SSH

mod common;

use common::*;
use std::fs::remove_file;

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_standard() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/test_ssh_standard/");

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_standard/.gitconfig.coliru
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_standard/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_standard/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc.coliru"));
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_alternate_tag_rules_1() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "linux", "^windows"]);
    copy_manifest(&dirs.local, "~/test_ssh_run_alternate_tag_rules_1/");

    let expected_stdout = format!("\
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_1/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_1/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux ^windows on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_exists = dirs.ssh.join(".gitconfig.coliru").exists();
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc.coliru"));
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_exists, false);
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux ^windows\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_alternate_tag_rules_2() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "macos"]);
    copy_manifest(&dirs.local, "~/test_ssh_run_alternate_tag_rules_2/");

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.gitconfig.coliru
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_alternate_tag_rules_2/.vimrc.coliru
[2/3] Run sh script.sh arg1 macos on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc.coliru"));
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
fn test_ssh_dry_run() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/test_ssh_dry_run/");

    let expected = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_dry_run/.gitconfig.coliru (DRY RUN)
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_dry_run/.bashrc.coliru (DRY RUN)
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_dry_run/.vimrc.coliru (DRY RUN)
[2/3] Run sh script.sh arg1 linux on {SSH_HOST} (DRY RUN)
");
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/run
    let bash_exists = dirs.ssh.join(".bashrc.coliru").exists();
    let git_exists = dirs.ssh.join(".gitconfig.coliru").exists();
    let vim1_exists = dirs.ssh.join(".vimrc.coliru").exists();
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    // let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    // assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_copy() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/test_ssh_copy/");

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_copy/.gitconfig.coliru
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_copy/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_copy/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc.coliru"));
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_run_failure() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/test_ssh_run_failure/");
    write_file(&dirs.local.join("script.sh"), "exit 1");

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_run_failure/.gitconfig.coliru
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_run_failure/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_run_failure/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.ssh.join(".vimrc.coliru"));
    let vim2_exists = dirs.ssh.join("_vimrc.coliru").exists();
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_ssh_missing_file() {
    let (dirs, mut cmd) = setup_e2e_ssh("test_ssh_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/test_ssh_missing_file/");
    remove_file(&dirs.local.join("vimrc")).unwrap();

    let expected_stdout = format!("\
[1/3] Copy gitconfig to {SSH_HOST}:~/test_ssh_missing_file/.gitconfig.coliru
[2/3] Copy bashrc to {SSH_HOST}:~/test_ssh_missing_file/.bashrc.coliru
[2/3] Copy vimrc to {SSH_HOST}:~/test_ssh_missing_file/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: No such file or directory (os error 2)
  Error: not implemented
";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/run
    let bash_contents = read_file(&dirs.ssh.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.ssh.join(".gitconfig.coliru"));
    // let log_contents = read_file(&dirs.ssh.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}
