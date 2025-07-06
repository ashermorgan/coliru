# coliru

A minimal, flexible, dotfile installer

[![asciicast](https://asciinema.org/a/680428.svg)](https://asciinema.org/a/680428)

## Features

With coliru you can:

- Install dotfiles as copies and/or symlinks
- Manage differences between machines using tags
- Run custom scripts
- Install dotfiles on remote machines over SSH

## Installation

Coliru binaries can be downloaded from the
[GitHub releases page](https://github.com/ashermorgan/coliru/releases).

Coliru can also be installed from source using Cargo:

```
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --git https://git.ashermorgan.net/coliru/

# To uninstall:
# cargo uninstall coliru
```

## Usage

Dotfile metadata is stored in a manifest file as a series of steps that can be
executed conditionally based on tag rules. To install dotfiles, pass the
location of the manifest file and the desired tag rules:

```
coliru manifest.yml --tag-rules tag1 tag2,tag3 ^tag4
```

Some other helpful options include:

- `--help`, `-h`: Print full help information
- `--list-tags`, `-l`: List the tags in the manifest and quit without installing
- `--dry-run`, `-n`: Do a trial run without any permanent changes
- `--host <HOST>`: Install dotfiles on another machine over SSH
- `--copy`: Interpret link commands as copy commands

### Manifest File

Manifests are defined using YAML as an array of steps that are executed to
install the dotfiles, which are located in the same directory as the manifest.
Each step may contain an array of copy, link, and/or run commands, in addition
to an array of tags (see below). Each command is run from the directory
containing the manifest file, or relative to the `~/.coliru` directory when
installing over SSH.

- The **copy** command copies a dotfile (`src`) to a destination (`dst`).
  Missing parent directories are created automatically.
- The **link** command links a dotfile (`src`) to a destination (`dst`) using
  symbolic links on Unix and hard links on Windows. Missing parent directories
  are created automatically and coliru will run copy commands in place
  of link commands when installing over SSH.
- The **run** command executes a script (`src`) from the command line, using
  `sh` on Unix and `cmd` on Windows, with an optional `prefix` (e.g. `python3`)
  or `postfix` (e.g. `arg1 arg2 arg3`) string. Inside `postfix`, `$COLIRU_RULES`
  will be expanded into a space-delimited list of the current tag rules. When
  installing over SSH, scripts are copied to the `~/.coliru` directory on the
  remote machine before they are executed.

Example YAML manifest (see `examples/basic/` for a complete example dotfile
repository):

```yml
steps:
  - copy:
    - src: gitconfig
      dst: ~/.gitconfig
    tags: [ windows, linux, macos ]

  - link:
    - src: bashrc
      dst: ~/.bashrc
    - src: vimrc
      dst: ~/.vimrc # Will create symbolic links on Linux & MacOS
    run:
    - src: ./script.sh
      prefix: sh # unecessary on Unix if script.sh is executable
      postfix: arg1 $COLIRU_RULES
    tags: [ linux, macos ]

  - link:
    - src: vimrc
      dst: ~/_vimrc # Will create hard link on Windows
    run:
    - src: script.bat
      postfix: arg1 $COLIRU_RULES
    tags: [ windows ]
```

### Tags and Tag Rules

Tags enable the installation of a subset of manifest steps based on a set of tag
rules. Tags are user-defined and may be used to specify a step's supported
operating systems (e.g. `linux`, `macos`, and/or `windows`), privilege
requirements (e.g. `system` vs `user`), or even the types of machines it applies
to (e.g. `personal`, `work`, `server`, etc).

Tag rules are specified on the command line using the `--tag-rules` option. In
order for the commands in a step to be executed, its tags must satisfy all of
the tag rules. If no tags rules are provided, all manifest steps will be
executed. Each tag rule contains a comma separated list of tags that can satisfy
the rule. A leading caret (`^`) will negate the entire rule.

In other words, commas correspond to OR, carets to NOT, and the spaces between
rules to AND. So `--tag-rules A B,C ^D,E` looks for steps with the tags `A && (B
|| C) && !(D || E)`.

## Development

Use Cargo to build, test, and run coliru:

```
cargo build
cargo test
cargo run -- --help
```

Some of coliru's integration and end-to-end tests interact with a test SSH
server, which can be started with Docker Compose:

```
docker compose -f tests/server/compose.yml up
```
