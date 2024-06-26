mod common;

use common::*;
use std::env::current_exe;
use std::fs::{copy, read_to_string, remove_file};
use std::path::Path;

/// Create a basic manifest file and its associated dotfiles in a directory
fn manifest_1(dir: &Path) {
    // Copy files from examples
    let examples = current_exe().unwrap().parent().unwrap().to_path_buf()
        .join("../../../examples");
    let copy_file = |name: &str| {
        copy(examples.join(name), &dir.join(name)).unwrap();
    };
    copy_file("script.bat");
    copy_file("script.sh");
    copy_file("manifest.yml");

    // Create simplified config files
    write_file(&dir.join("gitconfig"), "git #1");
    write_file(&dir.join("vimrc"), "vim #1");

}

#[test]
fn test_help() {
    let (_dir, mut cmd) = setup("test_help");
    cmd.arg("--help");
    let expected = "\
A minimal, flexible, dotfile installer

Usage: coliru [OPTIONS] <MANIFEST>

Arguments:
  <MANIFEST>  The path to the coliru YAML manifest file

Options:
  -t, --tag-rules [<TAG_RULES>...]  The set of tag rules to enforce
  -c, --copy                        Interpret link commands as copy commands
  -n, --dry-run                     Do a trial run without any permanent changes
  -h, --help                        Print help
  -V, --version                     Print version
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");
}

#[test]
#[cfg(target_os = "linux")]
fn test_standard() {
    let (dir, mut cmd) = setup("test_standard");
    cmd.args(["manifest.yml", "-t", "linux"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("gitconfig"), "git #2");
    write_file(&dir.dir.join("vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig.coliru")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc.coliru")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log.txt")).unwrap();
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
fn test_run_alternate_tag_rules_1() {
    let (dir, mut cmd) = setup("test_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "macos"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 macos
script.sh called with arg1 macos
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("gitconfig"), "git #2");
    write_file(&dir.dir.join("vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig.coliru")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc.coliru")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log.txt")).unwrap();
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 macos\n");
}

#[test]
#[cfg(target_os = "linux")]
fn test_run_alternate_tag_rules_2() {
    let (dir, mut cmd) = setup("test_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "linux,macos", "^windows"]);
    manifest_1(&dir.dir);

    let expected = "\
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux,macos ^windows
script.sh called with arg1 linux,macos ^windows
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("vimrc"), "vim #2");
    let git_exists = dir.dir.join(".gitconfig.coliru").exists();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc.coliru")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log.txt")).unwrap();
    assert_eq!(git_exists, false);
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux,macos ^windows\n");
}

#[test]
fn test_dry_run() {
    let (dir, mut cmd) = setup("test_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru (DRY RUN)
[2/3] Link vimrc to ~/.vimrc.coliru (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    let git_exists = dir.dir.join(".gitconfig.coliru").exists();
    let vim1_exists = dir.dir.join(".vimrc.coliru").exists();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_exists = dir.dir.join("log.txt").exists();
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_copy() {
    let (dir, mut cmd) = setup("test_copy");
    cmd.args(["manifest.yml", "--copy", "-t", "linux"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Copy vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("gitconfig"), "git #2");
    write_file(&dir.dir.join("vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig.coliru")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc.coliru")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log.txt")).unwrap();
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #1");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
#[cfg(target_os = "linux")]
fn test_run_failure() {
    let (dir, mut cmd) = setup("test_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    manifest_1(&dir.dir);
    write_file(&dir.dir.join("script.sh"), "exit 1");

    let expected_stdout = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
";
    let expected_stderr = "  Error: Process exited with exit status: 1\n";
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("gitconfig"), "git #2");
    write_file(&dir.dir.join("vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig.coliru")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc.coliru")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    assert_eq!(git_contents, "git #1");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_missing_file() {
    let (dir, mut cmd) = setup("test_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    manifest_1(&dir.dir);
    remove_file(&dir.dir.join("vimrc")).unwrap();

    let expected_stdout = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru
[2/3] Link vimrc to ~/.vimrc.coliru
[2/3] Run sh script.sh arg1 linux
script.sh called with arg1 linux
";
    let expected_stderr = "  Error: No such file or directory (os error 2)\n";
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("gitconfig"), "git #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig.coliru")).unwrap();
    let log_contents = read_to_string(&dir.dir.join("log.txt")).unwrap();
    assert_eq!(git_contents, "git #1");
    assert_eq!(log_contents, "script.sh called with arg1 linux\n");
}

#[test]
fn test_empty_manifest() {
    let (dir, mut cmd) = setup("test_empty_manifest");
    cmd.args(["manifest.yml"]);
    write_file(&dir.dir.join("manifest.yml"), "");

    let expected = "Error: missing field `steps`\n";
    assert_eq!(&stdout_to_string(&mut cmd), "");
    assert_eq!(&stderr_to_string(&mut cmd), expected);
}

#[test]
#[cfg(target_os = "linux")]
fn test_missing_manifest() {
    let (_dir, mut cmd) = setup("test_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: No such file or directory (os error 2)\n";
    assert_eq!(&stdout_to_string(&mut cmd), "");
    assert_eq!(&stderr_to_string(&mut cmd), expected);
}
