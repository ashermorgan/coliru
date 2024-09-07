//! End to end tests that test installation behavior on the local file system

mod test_utils;

use test_utils::*;
use std::fs::remove_file;

#[test]
#[cfg(target_family = "unix")]
fn test_local_standard() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);

    let expected = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Link bashrc to ~/.bashrc
[2/2] Link vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 linux
foo!
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("gitconfig"), "git #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_contents = read_file(&dirs.home.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_standard() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_standard");
    cmd.args(["manifest.yml", "-t", "windows"]);

    let expected = "\
[1/2] Copy gitconfig to .gitconfig
[2/2] Copy foo to foo
[2/2] Link vimrc to _vimrc
[2/2] Run  script.bat arg1 windows
foo!\r
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2\r\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\r\n");
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig"));
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc"));
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1\r\n");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2\r\n");
    assert_eq!(foo_contents, "foo!\r\n");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_alternate_tag_rules_1() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "linux", "^windows"]);

    let expected = "\
[1/1] Copy foo to foo
[1/1] Link bashrc to ~/.bashrc
[1/1] Link vimrc to ~/.vimrc
[1/1] Run sh script.sh arg1 linux ^windows
foo!
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_exists = dirs.home.join(".gitconfig").exists();
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(git_exists, false);
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux ^windows\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_alternate_tag_rules_2() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "macos"]);

    let expected = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Link bashrc to ~/.bashrc
[2/2] Link vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 macos
foo!
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("gitconfig"), "git #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_contents = read_file(&dirs.home.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_dry_run() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);

    let expected = "\
[1/2] Copy gitconfig to ~/.gitconfig (DRY RUN)
[2/2] Copy foo to foo (DRY RUN)
[2/2] Link bashrc to ~/.bashrc (DRY RUN)
[2/2] Link vimrc to ~/.vimrc (DRY RUN)
[2/2] Run sh script.sh arg1 linux (DRY RUN)
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.home.join(".bashrc").exists();
    let git_exists = dirs.home.join(".gitconfig").exists();
    let vim1_exists = dirs.home.join(".vimrc").exists();
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_exists = dirs.local.join("foo").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_exists, true);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_dry_run() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "windows"]);

    let expected = "\
[1/2] Copy gitconfig to .gitconfig (DRY RUN)
[2/2] Copy foo to foo (DRY RUN)
[2/2] Link vimrc to _vimrc (DRY RUN)
[2/2] Run  script.bat arg1 windows (DRY RUN)
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_exists = dirs.local.join(".gitconfig").exists();
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_exists = dirs.local.join("_vimrc").exists();
    let foo_exists = dirs.local.join("foo").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_exists, true);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_copy() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);

    let expected = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Copy bashrc to ~/.bashrc
[2/2] Copy vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 linux
foo!
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("gitconfig"), "git #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_contents = read_file(&dirs.home.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #1\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #1\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_copy() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "windows"]);

    let expected = "\
[1/2] Copy gitconfig to .gitconfig
[2/2] Copy foo to foo
[2/2] Copy vimrc to _vimrc
[2/2] Run  script.bat arg1 windows
foo!\r
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2\r\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\r\n");
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig"));
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc"));
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1\r\n");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #1\r\n");
    assert_eq!(foo_contents, "foo!\r\n");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_run_failure() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    write_file(&dirs.local.join("script.sh"), "exit 1");

    let expected_stdout = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Link bashrc to ~/.bashrc
[2/2] Link vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 linux
";
    let expected_stderr = "  Error: Process terminated unsuccessfully: \
                           exit status: 1\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("gitconfig"), "git #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_contents = read_file(&dirs.home.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_run_failure() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_run_failure");
    cmd.args(["manifest.yml", "-t", "windows"]);
    write_file(&dirs.local.join("script.bat"), "@echo off\r\nexit 1");

    let expected_stdout = "\
[1/2] Copy gitconfig to .gitconfig
[2/2] Copy foo to foo
[2/2] Link vimrc to _vimrc
[2/2] Run  script.bat arg1 windows
";
    let expected_stderr = "  Error: Process terminated unsuccessfully: \
                           exit code: 1\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2\r\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\r\n");
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig"));
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc"));
    let foo_contents = read_file(&dirs.local.join("foo"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1\r\n");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2\r\n");
    assert_eq!(foo_contents, "foo!\r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_missing_file() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    remove_file(&dirs.local.join("gitconfig")).unwrap();

    let expected_stdout = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Link bashrc to ~/.bashrc
[2/2] Link vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 linux
foo!
";
    let expected_stderr = "  Error: No such file or directory (os error 2)\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_missing_file() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_missing_file");
    cmd.args(["manifest.yml", "-t", "windows"]);
    remove_file(&dirs.local.join("vimrc")).unwrap();

    let expected_stdout = "\
[1/2] Copy gitconfig to .gitconfig
[2/2] Copy foo to foo
[2/2] Link vimrc to _vimrc
[2/2] Run  script.bat arg1 windows
foo!\r
";
    let expected_stderr = "  Error: The system cannot find the file specified. \
                           (os error 2)\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected_stderr);
    assert_eq!(&stdout, expected_stdout);
    assert_eq!(exitcode, Some(1));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2\r\n");
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig"));
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1\r\n");
    assert_eq!(foo_contents, "foo!\r\n");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}

#[test]
#[cfg(target_family = "unix")]
fn test_local_relative_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_relative_manifest");
    cmd.current_dir(&dirs.local.parent().unwrap());
    cmd.args(["test_local_relative_manifest/manifest.yml", "-t", "linux"]);

    let expected = "\
[1/2] Copy gitconfig to ~/.gitconfig
[2/2] Copy foo to foo
[2/2] Link bashrc to ~/.bashrc
[2/2] Link vimrc to ~/.vimrc
[2/2] Run sh script.sh arg1 linux
foo!
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("bashrc"), "bash #2\n");
    write_file(&dirs.local.join("gitconfig"), "git #2\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\n");
    let bash_contents = read_file(&dirs.home.join(".bashrc"));
    let git_contents = read_file(&dirs.home.join(".gitconfig"));
    let vim1_contents = read_file(&dirs.home.join(".vimrc"));
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_contents, "bash #2\n");
    assert_eq!(git_contents, "git #1\n");
    assert_eq!(vim1_contents, "vim #2\n");
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_contents, "foo!\n");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_family = "windows")]
fn test_local_different_cwd() {
    let (dirs, mut cmd) = setup_e2e_local("test_local_different_cwd");
    cmd.current_dir(&dirs.local.parent().unwrap());
    cmd.args(["test_local_different_cwd/manifest.yml", "-t", "windows"]);

    let expected = "\
[1/2] Copy gitconfig to .gitconfig
[2/2] Copy foo to foo
[2/2] Link vimrc to _vimrc
[2/2] Run  script.bat arg1 windows
foo!\r
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    write_file(&dirs.local.join("gitconfig"), "git #2\r\n");
    write_file(&dirs.local.join("vimrc"), "vim #2\r\n");
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_contents = read_file(&dirs.local.join(".gitconfig"));
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_contents = read_file(&dirs.local.join("_vimrc"));
    let foo_contents = read_file(&dirs.local.join("foo"));
    let log_contents = read_file(&dirs.local.join("log.txt"));
    assert_eq!(bash_exists, false);
    assert_eq!(git_contents, "git #1\r\n");
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2\r\n");
    assert_eq!(foo_contents, "foo!\r\n");
    assert_eq!(log_contents, "script.bat called with arg1 windows \r\n");
}
