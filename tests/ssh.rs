/// End to end tests that test specific installation behavior on a remote
/// machine via SSH

mod common;

use common::*;
use std::fs::remove_file;

#[test]
fn test_ssh_standard() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dir.dir);

    let expected_stdout = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("gitconfig"), "git #2");
    // write_file(&dir.dir.join("vimrc"), "vim #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_contents = read_file(&dir.dir.join(".gitconfig.coliru"));
    // let vim1_contents = read_file(&dir.dir.join(".vimrc.coliru"));
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dir.dir.join("log.txt"));
    // assert_eq!(bash_contents, "bash #2");
    // assert_eq!(git_contents, "git #1");
    // assert_eq!(vim1_contents, "vim #2");
    // assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
fn test_ssh_run_alternate_tag_rules_1() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "linux", "^windows"]);
    copy_manifest(&dir.dir);

    let expected_stdout = format!("\
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux ^windows on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("vimrc"), "vim #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_exists = dir.dir.join(".gitconfig.coliru").exists();
    // let vim1_contents = read_file(&dir.dir.join(".vimrc.coliru"));
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dir.dir.join("log.txt"));
    // assert_eq!(bash_contents, "bash #2");
    // assert_eq!(git_exists, false);
    // assert_eq!(vim1_contents, "vim #2");
    // assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux ^windows\n");
}

#[test]
fn test_ssh_run_alternate_tag_rules_2() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "macos"]);
    copy_manifest(&dir.dir);

    let expected_stdout = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 macos on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("gitconfig"), "git #2");
    // write_file(&dir.dir.join("vimrc"), "vim #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_contents = read_file(&dir.dir.join(".gitconfig.coliru"));
    // let vim1_contents = read_file(&dir.dir.join(".vimrc.coliru"));
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dir.dir.join("log.txt"));
    // assert_eq!(bash_contents, "bash #2");
    // assert_eq!(git_contents, "git #1");
    // assert_eq!(vim1_contents, "vim #2");
    // assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
fn test_ssh_dry_run() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);
    copy_manifest(&dir.dir);

    let expected = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru (DRY RUN)
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru (DRY RUN)
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru (DRY RUN)
[2/3] Run sh script.sh arg1 linux on {SSH_HOST} (DRY RUN)
");
    assert_eq!(stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    // let bash_exists = dir.dir.join(".bashrc.coliru").exists();
    // let git_exists = dir.dir.join(".gitconfig.coliru").exists();
    // let vim1_exists = dir.dir.join(".vimrc.coliru").exists();
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // let log_exists = dir.dir.join("log.txt").exists();
    // assert_eq!(bash_exists, false);
    // assert_eq!(git_exists, false);
    // assert_eq!(vim1_exists, false);
    // assert_eq!(vim2_exists, false);
    // assert_eq!(log_exists, false);
}

#[test]
fn test_ssh_copy() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);
    copy_manifest(&dir.dir);

    let expected_stdout = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("gitconfig"), "git #2");
    // write_file(&dir.dir.join("vimrc"), "vim #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_contents = read_file(&dir.dir.join(".gitconfig.coliru"));
    // let vim1_contents = read_file(&dir.dir.join(".vimrc.coliru"));
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // let log_contents = read_file(&dir.dir.join("log.txt"));
    // assert_eq!(bash_contents, "bash #1");
    // assert_eq!(git_contents, "git #1");
    // assert_eq!(vim1_contents, "vim #1");
    // assert_eq!(vim2_exists, false);
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
fn test_ssh_run_failure() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dir.dir);
    write_file(&dir.dir.join("script.sh"), "exit 1");

    let expected_stdout = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("gitconfig"), "git #2");
    // write_file(&dir.dir.join("vimrc"), "vim #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_contents = read_file(&dir.dir.join(".gitconfig.coliru"));
    // let vim1_contents = read_file(&dir.dir.join(".vimrc.coliru"));
    // let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    // assert_eq!(bash_contents, "bash #2");
    // assert_eq!(git_contents, "git #1");
    // assert_eq!(vim1_contents, "vim #2");
    // assert_eq!(vim2_exists, false);
}

#[test]
fn test_ssh_missing_file() {
    let (dir, mut cmd) = setup_e2e_ssh("test_ssh_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dir.dir);
    remove_file(&dir.dir.join("vimrc")).unwrap();

    let expected_stdout = format!("\
[1/3] Send gitconfig to {SSH_HOST}:~/.gitconfig.coliru
[2/3] Send bashrc to {SSH_HOST}:~/.bashrc.coliru
[2/3] Send vimrc to {SSH_HOST}:~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux on {SSH_HOST}
");
    let expected_stderr = "  Error: not implemented
  Error: not implemented
  Error: not implemented
  Error: not implemented
";
    assert_eq!(stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    // write_file(&dir.dir.join("bashrc"), "bash #2");
    // write_file(&dir.dir.join("gitconfig"), "git #2");
    // let bash_contents = read_file(&dir.dir.join(".bashrc.coliru"));
    // let git_contents = read_file(&dir.dir.join(".gitconfig.coliru"));
    // let log_contents = read_file(&dir.dir.join("log.txt"));
    // assert_eq!(bash_contents, "bash #2");
    // assert_eq!(git_contents, "git #1");
    // assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}
