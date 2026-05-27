# PROGRESS

## 2026-05-27
- Pointed the Python console script at `cli_entrypoint` instead of the removed `main` export.
- Made the Python console entrypoint return process exit codes without raising tracebacks for CLI failures.
- Fixed clippy warning in PyO3 module export list construction.

## 2026-05-26
- Hardened branch detection to return errors for git failures and only skip on detached HEAD.
- Switched commit parsing to NUL-delimited `git log` output to avoid delimiter-confusion issues.
- Sanitized terminal output fields to reduce control-sequence injection risk.
- Removed Python-exported CLI `main` entrypoint that could terminate embedding processes.
- Replaced tautological config test with deterministic assertion.
