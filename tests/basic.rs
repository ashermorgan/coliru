mod common;

use common::*;
use std::fs::{create_dir_all, read_to_string, remove_file};
use std::path::Path;

/// Create a basic manifest file and its associated dotfiles in a directory
fn manifest_1(dir: &Path) {
    let manifest = "\
steps:
  - copy:
    - src: git_dotfiles/.gitconfig
      dst: ~/.gitconfig
    link:
    - src: vim_dotfiles/.vimrc
      dst: ~/.vimrc
    tags: [ windows, linux, macos ]
  - link:
    - src: vim_dotfiles/.vimrc
      dst: ~/_vimrc
    tags: [ windows ]
  - run:
    - src: install_programs.sh
      prefix: bash # Unecessary if install_programs.sh is executable
      postfix: $COLIRU_RULES -y
    tags: [ linux ]
";
    write_file(&dir.join("manifest.yml"), manifest);
    create_dir_all(&dir.join("git_dotfiles")).unwrap();
    write_file(&dir.join("git_dotfiles").join(".gitconfig"), "git config");
    create_dir_all(&dir.join("vim_dotfiles")).unwrap();
    write_file(&dir.join("vim_dotfiles").join(".vimrc"), "vim config");
    write_file(&dir.join("install_programs.sh"), "echo $@ > log");
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
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig
[1/3] Link vim_dotfiles/.vimrc to ~/.vimrc
[3/3] Run bash install_programs.sh linux -y
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("git_dotfiles").join(".gitconfig"), "git #2");
    write_file(&dir.dir.join("vim_dotfiles").join(".vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log")).unwrap();
    assert_eq!(git_contents, "git config");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "linux -y\n");
}

#[test]
fn test_run_alternate_tag_rules_1() {
    let (dir, mut cmd) = setup("test_run_alternate_tag_rules_1");
    cmd.args(["manifest.yml", "-t", "macos"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig
[1/3] Link vim_dotfiles/.vimrc to ~/.vimrc
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("git_dotfiles").join(".gitconfig"), "git #2");
    write_file(&dir.dir.join("vim_dotfiles").join(".vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_exists = dir.dir.join("log").exists();
    assert_eq!(git_contents, "git config");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_run_alternate_tag_rules_2() {
    let (dir, mut cmd) = setup("test_run_alternate_tag_rules_2");
    cmd.args(["manifest.yml", "-t", "linux,windows", "^macos"]);
    manifest_1(&dir.dir);

    let expected = "\
[2/3] Link vim_dotfiles/.vimrc to ~/_vimrc
[3/3] Run bash install_programs.sh linux,windows ^macos -y
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("vim_dotfiles").join(".vimrc"), "vim #2");
    let git_exists = dir.dir.join(".gitconfig").exists();
    let vim1_exists = dir.dir.join(".vimrc").exists();
    let vim2_contents = read_to_string(&dir.dir.join("_vimrc")).unwrap();
    let log_contents = read_to_string(&dir.dir.join("log")).unwrap();
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_contents, "vim #2");
    assert_eq!(log_contents, "linux,windows ^macos -y\n");
}

#[test]
fn test_dry_run() {
    let (dir, mut cmd) = setup("test_dry_run");
    cmd.args(["manifest.yml", "--dry-run", "-t", "linux"]);
    manifest_1(&dir.dir);

    let expected = "\
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig (DRY RUN)
[1/3] Link vim_dotfiles/.vimrc to ~/.vimrc (DRY RUN)
[3/3] Run bash install_programs.sh linux -y (DRY RUN)
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    let git_exists = dir.dir.join(".gitconfig").exists();
    let vim1_exists = dir.dir.join(".vimrc").exists();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_exists = dir.dir.join("log").exists();
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
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig
[1/3] Copy vim_dotfiles/.vimrc to ~/.vimrc
[3/3] Run bash install_programs.sh linux -y
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("git_dotfiles").join(".gitconfig"), "git #2");
    write_file(&dir.dir.join("vim_dotfiles").join(".vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    let log_contents = read_to_string(&dir.dir.join("log")).unwrap();
    assert_eq!(git_contents, "git config");
    assert_eq!(vim1_contents, "vim config");
    assert_eq!(vim2_exists, false);
    assert_eq!(log_contents, "linux -y\n");
}

#[test]
#[cfg(target_os = "linux")]
fn test_run_failure() {
    let (dir, mut cmd) = setup("test_run_failure");
    cmd.args(["manifest.yml", "-t", "linux"]);
    manifest_1(&dir.dir);
    write_file(&dir.dir.join("install_programs.sh"), "exit 1");

    let expected_stdout = "\
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig
[1/3] Link vim_dotfiles/.vimrc to ~/.vimrc
[3/3] Run bash install_programs.sh linux -y
";
    let expected_stderr = "  Error: Process exited with exit status: 1\n";
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("git_dotfiles").join(".gitconfig"), "git #2");
    write_file(&dir.dir.join("vim_dotfiles").join(".vimrc"), "vim #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig")).unwrap();
    let vim1_contents = read_to_string(&dir.dir.join(".vimrc")).unwrap();
    let vim2_exists = dir.dir.join("_vimrc").exists();
    assert_eq!(git_contents, "git config");
    assert_eq!(vim1_contents, "vim #2");
    assert_eq!(vim2_exists, false);
}

#[test]
#[cfg(target_os = "linux")]
fn test_missing_file() {
    let (dir, mut cmd) = setup("test_missing_file");
    cmd.args(["manifest.yml", "-t", "linux"]);
    manifest_1(&dir.dir);
    remove_file(&dir.dir.join("vim_dotfiles").join(".vimrc")).unwrap();

    let expected_stdout = "\
[1/3] Copy git_dotfiles/.gitconfig to ~/.gitconfig
[1/3] Link vim_dotfiles/.vimrc to ~/.vimrc
[3/3] Run bash install_programs.sh linux -y
";
    let expected_stderr = "  Error: No such file or directory (os error 2)\n";
    assert_eq!(&stdout_to_string(&mut cmd), expected_stdout);
    assert_eq!(&stderr_to_string(&mut cmd), expected_stderr);

    // Assert files are correctly copied/linked/run
    write_file(&dir.dir.join("git_dotfiles").join(".gitconfig"), "git #2");
    let git_contents = read_to_string(&dir.dir.join(".gitconfig")).unwrap();
    let log_contents = read_to_string(&dir.dir.join("log")).unwrap();
    assert_eq!(git_contents, "git config");
    assert_eq!(log_contents, "linux -y\n");
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
