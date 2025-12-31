# Mixed Pickles

A fast git commit message linter written in Rust. Validates commit messages against best practices and provides actionable suggestions.

## Installation

### Python (Recommended)

```bash
pip install mixed-pickles
```

Or with [uv](https://github.com/astral-sh/uv):

```bash
uv pip install mixed-pickles
```

### Rust

```bash
cargo install mixed-pickles
```

Or build from source:

```bash
git clone https://github.com/rommeld/mixed-pickles.git
cd mixed-pickles
cargo build --release
```

## What It Checks

| Validation         | Default Severity | Description                                  |
| ------------------ | ---------------- | -------------------------------------------- |
| `WipCommit`        | Error            | WIP markers, fixup!, squash!                 |
| `ShortCommit`      | Warning          | Messages below threshold (default: 30 chars) |
| `NonImperative`    | Warning          | Non-imperative mood ("Added" vs "Add")       |
| `VagueLanguage`    | Warning          | Generic phrases ("fix bug", "update code")   |
| `MissingReference` | Info             | No issue reference (#123, PROJ-456)          |
| `InvalidFormat`    | Info             | Not following conventional commits           |

## Usage

### CLI

```bash
# Analyze all commits in current repo
mixed-pickles

# Analyze last 10 commits
mixed-pickles --limit 10

# Analyze specific repo
mixed-pickles --path /path/to/repo

# Strict mode (warnings become errors)
mixed-pickles --strict

# Customize severity
mixed-pickles --error short,vague --ignore ref

# Quiet mode (output only on issues)
mixed-pickles --quiet
```

### Python API

```python
import mixed_pickles

# Basic analysis - auto-loads pyproject.toml config
mixed_pickles.analyze_commits()

# Analyze with options
mixed_pickles.analyze_commits(
    path=".",           # Repository path
    limit=10,           # Number of commits
    quiet=True,         # Suppress output unless issues
    strict=True         # Treat warnings as errors
)

# Disable auto-loading of config file
mixed_pickles.analyze_commits(use_config=False)

# Load config from pyproject.toml or .mixed-pickles.toml
config = mixed_pickles.ValidationConfig.discover()
config = mixed_pickles.ValidationConfig.discover("/path/to/project")

# Load config from specific file
config = mixed_pickles.ValidationConfig.from_file("pyproject.toml")
config = mixed_pickles.ValidationConfig.from_file(".mixed-pickles.toml")

# Manual configuration
config = mixed_pickles.ValidationConfig(
    threshold=50,                    # Minimum message length
    require_issue_ref=False,         # Disable issue reference check
    require_conventional_format=False,
    check_vague_language=True,
    check_wip=True,
    check_imperative=True
)
mixed_pickles.analyze_commits(config=config)

# Adjust severity levels
config = mixed_pickles.ValidationConfig()
config.set_severity(
    mixed_pickles.Validation.MissingReference,
    mixed_pickles.Severity.Error
)
config.set_severity(
    mixed_pickles.Validation.ShortCommit,
    mixed_pickles.Severity.Ignore
)
mixed_pickles.analyze_commits(config=config)

# Fetch commits for custom processing
commits = mixed_pickles.fetch_commits(limit=5)
for commit in commits:
    print(f"{commit.short_hash}: {commit.message}")
```

### Pre-commit Hook

Add to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: mixed-pickles
        name: Validate commit messages
        entry: mixed-pickles
        language: python
        additional_dependencies: [mixed-pickles]
        always_run: true
        pass_filenames: false
        stages: [pre-push]
```

With uv, you can also run it directly:

```yaml
repos:
  - repo: local
    hooks:
      - id: mixed-pickles
        name: Validate commit messages
        entry: uv run mixed-pickles
        language: system
        always_run: true
        pass_filenames: false
        stages: [pre-push]
```

## CLI Options

| Option             | Description                                  |
| ------------------ | -------------------------------------------- |
| `--path <PATH>`    | Repository path (default: current directory) |
| `--limit <N>`      | Max commits to analyze                       |
| `--threshold <N>`  | Minimum message length (default: 30)         |
| `--quiet`, `-q`    | Suppress output unless issues found          |
| `--strict`         | Treat warnings as errors                     |
| `--error <LIST>`   | Validations to treat as errors               |
| `--warn <LIST>`    | Validations to treat as warnings             |
| `--ignore <LIST>`  | Validations to skip reporting                |
| `--disable <LIST>` | Validations to disable entirely              |
| `--config <PATH>`  | Path to configuration file                   |
| `--no-config`      | Ignore configuration file                    |

Validation aliases for CLI: `short`, `ref`, `format`, `vague`, `wip`, `imperative`

## Configuration

Configure mixed-pickles via `pyproject.toml` for project-specific settings:

```toml
[tool.mixed-pickles]
# Minimum commit message length (default: 30)
threshold = 50

# Disable specific validations entirely
disable = ["reference", "format"]

# Override severity levels
[tool.mixed-pickles.severity]
wip = "error"          # Block on WIP commits (default)
short = "warning"      # Warn but allow (default)
vague = "ignore"       # Don't report
reference = "info"     # Informational only (default)
```

Or use a dedicated `.mixed-pickles.toml` file (takes precedence over `pyproject.toml`):

```toml
threshold = 50
disable = ["format"]

[severity]
short = "error"
```

### Configuration Precedence

Settings are applied in this order (later overrides earlier):

1. **Defaults** - Built-in default values
2. **Config file** - `pyproject.toml` or `.mixed-pickles.toml`
3. **CLI arguments** - Command-line flags

### Available Validations

| Name | Aliases | Default | Description |
|------|---------|---------|-------------|
| `short` | `short-commit` | warning | Message below threshold |
| `wip` | `wip-commit` | error | WIP/fixup/squash markers |
| `reference` | `ref`, `missing-reference` | info | Missing issue reference |
| `format` | `invalid-format` | info | Not conventional commits |
| `vague` | `vague-language` | warning | Generic descriptions |
| `imperative` | `non-imperative` | warning | Past/continuous tense |

## Severity Levels

- **Error**: Fails the check (exit code 1)
- **Warning**: Reported but passes (fails with `--strict`)
- **Info**: Informational only
- **Ignore**: Tracked but not reported

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.
