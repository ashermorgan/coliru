# Changelog

## 0.2.0 - 2024-06-25

### Added

- `link` command to create soft/hard links with `--copy` flag to fallback to
  `copy` command
- `run` command to run scripts

### Changed

- Improved output formatting

### Fixed

- Bug where `--dry-run` flag didn't print all commands
- Issue causing Windows builds to fail
- Bug that prevented manifests in the current directory from being loaded

## 0.1.0 - 2024-06-20

### Added

- Support for manifests, including `copy` command
- Steps can be filtered using basic tag rules
- `--dry-run` flag
