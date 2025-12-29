# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- add NonImperative validation to detect commit messages that do not use imperative mood (7302698)
- add WIPCommit validation to detect wip commits (671e865)
- add configurable severity levels for commit validations (d947f21)
- add --strict flag to treat warnings as errors (b60e345)
- show total repository commit count in output (1284416)

### Changed

- improve CLI output format with better commit details (b60e345)
- simplify analyze_commits Python API (2df7e76)

### Fixed

- handle singular/plural in commit issue messages (18b60cf, 48f4683)

## [0.4.0] - 2025-12-29

### Added

- add MissingReference, InvalidFormat, and VagueLanguage validation to detect generic commit descriptions (e145fd3)

## [0.3.0] - 2025-12-26

### Added

- expose 'Commit' struct (b429346)
- add pyfunction 'analyze_commit' for Python integration (42bb51d)

## [0.2.2] - 2025-12-23

### Changed

- reduce number of validated commits to cli --limit (6bec626)

## [0.2.1] - 2025-12-23

### Fixed

- delete reference in functions and types (5dcf168)

### Added

- add validation for creating Commit struct from git log (4cd9f9d)

## [0.2.0] - 2025-12-23

### Added

- add configurable commit analysis options (f7491a0)
