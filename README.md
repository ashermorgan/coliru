# coliru
A minimal, flexible, dotfile installer

## Installation
To install `coliru`, clone the repository and run `cargo install --path coliru/`

To uninstall `coliru`, run `cargo uninstall coliru`

## Usage
Dotfiles are defined as a series of steps inside a manifest file that are
executed conditionally based on tag rules. To install dotfiles, pass the
manifest file and tag rules to `coliru`:

```
coliru path/to/manifest.yml --tag-rules tag1 tag2,tag3 ^tag4
```

Some helpful options include:

- `--help`, `-h`: Print full help information
- `--dry-run`, `-n`: Do a trial run without any permanent changes
- `--copy`, `-c`: Interpret link commands as copy commands

## Manifest File
Manifests are defined using YAML. Each manifest contains a series of steps that
are executed to install the dotfiles. Each step contains an array of tags and
any number of copy, link, or run commands. Each command is run from the
directory containing the manifest file. The copy command copies a file from a
source (`src`) to a (`dst`). The link command links a file from a source (`src`)
to a (`dst`) using symbolic links on Unix platforms and hard links on Windows.
Finally, the run command executes a script (`src`) from the command line, using
`sh` on Unix platforms and `powershell` on Windows, with an optional `prefix`
(e.g. `python3`) or `postfix` (e.g. `arg1 arg2 arg3`) string. Inside `postfix`,
`$COLIRU_RULES` will be expanded into a space-delimited list of the current tag
rules.

Example YAML manifest (see [`examples/manifest.yml`](examples/manifest.yml) for
more details):

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
    - src: script.sh
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

## Tag Rules
Tag rules are specified on the command line using the `--tag-rules` option. In
order for a manifest step to be executed, its tags must satisfy all of the tag
rules. If no tags rules are provided, all manifest steps will be executed. Each
tag rule contains a comma separated list of tags that will satisfy the rule. A
leading caret (`^`) will negate the entire rule. In other words, commas
correspond to OR, carets to NOT, and the spaces between rules to AND. So the tag
rules `A B,C ^D,E` are equivalent to `A && (B || C) && !(D || E)`. Common tags
include supported operating systems and whether the step requires elevated
privileges.
