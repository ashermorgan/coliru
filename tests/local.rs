/// End to end tests that test specific installation behavior on the local file
/// system

mod common;

use common::*;
use std::fs::remove_file;

#[test]
#[cfg(target_family = "unix")]
fn test_local_standard() {
    let (dirs, mut cmd) = setup_e2e("test_local_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/");

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link bashrc to ~/.bashrc.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.home.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc.coliru"));
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_standard() {
    let (dirs, mut cmd) = setup_e2e("test_local_standard");
    cmd.args(["manifest.yml", "-t", "windows"]);
    copy_manifest(&dirs.local, "");

    let expected = "\
[1/3] Copy gitconfig to .gitconfig.coliru
[3/3] Link vimrc to _vimrc.coliru
[3/3] Run  script.bat arg1 windows
script.bat called with arg1 windows\r
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_exists = dirs.local.join(".bashrc.coliru").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig.coliru"));
    let vim1_exists = dirs.local.join(".vimrc.coliru").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc.coliru"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_alternate_tag_rules_1() {
    let (dirs, mut cmd) = setup_e2e("test_local_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "linux", "^windows"]);
    copy_manifest(&dirs.local, "~/");

    let expected = "\
[2/3] Link bashrc to ~/.bashrc.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux ^windows
script.sh called with arg1 linux ^windows
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_exists = dirs.home.join(".gitconfig.coliru").exists();
    let vim1_contents = read_file(&dirs.home.join(".vimrc.coliru"));
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2");
    assert_eq!(git_exists, false);
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux ^windows\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_alternate_tag_rules_2() {
    let (dirs, mut cmd) = setup_e2e("test_local_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "macos"]);
    copy_manifest(&dirs.local, "~/");

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link bashrc to ~/.bashrc.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 macos
script.sh called with arg1 macos
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.home.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc.coliru"));
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_dry_run() {
    let (dirs, mut cmd) = setup_e2e("test_local_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/");

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru (DRY RUN)
[2/3] Link bashrc to ~/.bashrc.coliru (DRY RUN)
[2/3] Link vimrc to ~/.vimrc.coliru (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.home.join(".bashrc.coliru").exists();
    let git_exists = dirs.home.join(".gitconfig.coliru").exists();
    let vim1_exists = dirs.home.join(".vimrc.coliru").exists();
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_dry_run() {
    let (dirs, mut cmd) = setup_e2e("test_local_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "windows"]);
    copy_manifest(&dirs.local, "");

    let expected = "\
[1/3] Copy gitconfig to .gitconfig.coliru (DRY RUN)
[3/3] Link vimrc to _vimrc.coliru (DRY RUN)
[3/3] Run  script.bat arg1 windows (DRY RUN)
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.local.join(".bashrc.coliru").exists();
    let git_exists = dirs.local.join(".gitconfig.coliru").exists();
    let vim1_exists = dirs.local.join(".vimrc.coliru").exists();
    let vim2_exists = dirs.local.join("_vimrc.coliru").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_copy() {
    let (dirs, mut cmd) = setup_e2e("test_local_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/");

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Copy bashrc to ~/.bashrc.coliru
[2/3] Copy vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.home.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc.coliru"));
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_copy() {
    let (dirs, mut cmd) = setup_e2e("test_local_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "windows"]);
    copy_manifest(&dirs.local, "");

    let expected = "\
[1/3] Copy gitconfig to .gitconfig.coliru
[3/3] Copy vimrc to _vimrc.coliru
[3/3] Run  script.bat arg1 windows
script.bat called with arg1 windows\r
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_exists = dirs.local.join(".bashrc.coliru").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig.coliru"));
    let vim1_exists = dirs.local.join(".vimrc.coliru").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc.coliru"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #1");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_failure() {
    let (dirs, mut cmd) = setup_e2e("test_local_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/");
    write_file(&dirs.local.join("script.sh"), "exit 1");

    let expected_stdout = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link bashrc to ~/.bashrc.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
";
    let expected_stderr = "  Error: Process exited with exit status: 1\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.home.join(".gitconfig.coliru"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc.coliru"));
    let vim2_exists = dirs.home.join("_vimrc.coliru").exists();
    assert_eq!(bash_contents, "bash #2");
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_run_failure() {
    let (dirs, mut cmd) = setup_e2e("test_local_run_failure");
    cmd.args(["manifest.yml", "-t", "windows"]);
    copy_manifest(&dirs.local, "");
    write_file(&dirs.local.join("script.bat"), "@echo off\r\nexit 1");

    let expected_stdout = "\
[1/3] Copy gitconfig to .gitconfig.coliru
[3/3] Link vimrc to _vimrc.coliru
[3/3] Run  script.bat arg1 windows
";
    let expected_stderr = "  Error: Process exited with exit code: 1\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2");
    write_file(&dirs.local.join("vimrc"), "vim #2");
    let bash_exists = dirs.local.join(".bashrc.coliru").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig.coliru"));
    let vim1_exists = dirs.local.join(".vimrc.coliru").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc.coliru"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_missing_file() {
    let (dirs, mut cmd) = setup_e2e("test_local_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    copy_manifest(&dirs.local, "~/");
    remove_file(&dirs.local.join("vimrc")).unwrap();

    let expected_stdout = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link bashrc to ~/.bashrc.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    let expected_stderr = "  Error: No such file or directory (os error 2)\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2");
    write_file(&dirs.local.join("gitconfig"), "git #2");
    let bash_contents = read_file(&dirs.home.join(".bashrc.coliru"));
    let git_contents = read_file(&dirs.home.join(".gitconfig.coliru"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2");
    assert_eq!(git_contents, "git #1");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_missing_file() {
    let (dirs, mut cmd) = setup_e2e("test_local_missing_file");
    cmd.args(["manifest.yml", "-t", "windows"]);
    copy_manifest(&dirs.local, "");
    remove_file(&dirs.local.join("vimrc")).unwrap();

    let expected_stdout = "\
[1/3] Copy gitconfig to .gitconfig.coliru
[3/3] Link vimrc to _vimrc.coliru
[3/3] Run  script.bat arg1 windows
script.bat called with arg1 windows\r
";
    let expected_stderr = "  Error: The system cannot find the file specified. \
                           (os error 2)\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2");
    let bash_exists = dirs.local.join(".bashrc.coliru").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig.coliru"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}
