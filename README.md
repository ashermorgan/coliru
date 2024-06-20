# coliru
A minimal, flexible, dotfile installer

Dotfiles are defined as a series of steps inside a manifest file, which are
    executed conditionally based on tag rules.
An example manifest file is located at `examples/2.yml` and the required YAML
    structure is defined in `src/manifest.rs`.
Tag rules follow the format described in `src/tags.rs`.
