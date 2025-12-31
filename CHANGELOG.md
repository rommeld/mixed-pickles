# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.0] - 2025-12-31

### Documentation

- add installation and usage to README.md (35f2127)

### Added

- add PyPI publishing workflow and package metadata (f5ab923)
- add pyproject.toml configuration support (8fd491d)
- add NonImperative validation to detect commit messages that do not use imperative mood (7302698)
- add WIPCommit validation to detect wip commits (671e865)
- add configurable severity levels for commit validations (d947f21)
- add --strict flag to treat warnings as errors (b60e345)
- show total repository commit count in output (1284416)
- add MissingReference, InvalidFormat, and VagueLanguage validation to detect generic commit descriptions (e145fd3)
- expose 'Commit' struct for Python integration (b429346)
- add pyfunction 'analyze_commit' for Python integration (42bb51d)
- add validation for creating Commit struct from git log (4cd9f9d)
- add configurable commit analysis options (f7491a0)

### Changed

- improve idiomatic Rust patterns in config handling (69f3dbb)
- improve CLI output format with better commit details (b60e345)
- simplify analyze_commits Python API (2df7e76)
- reduce number of validated commits to cli --limit (6bec626)

### Fixed

- run tests before publishing to PyPI (504616e)
- use has_analyzing_header helper to fix clippy (e4c4ee3)
- handle singular/plural in commit issue messages (18b60cf, 48f4683)
- delete reference in functions and types (5dcf168)
