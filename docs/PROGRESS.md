# PROGRESS

## 2026-05-26
- Hardened branch detection to return errors for git failures and only skip on detached HEAD.
- Switched commit parsing to NUL-delimited `git log` output to avoid delimiter-confusion issues.
- Sanitized terminal output fields to reduce control-sequence injection risk.
- Removed Python-exported CLI `main` entrypoint that could terminate embedding processes.
- Replaced tautological config test with deterministic assertion.
