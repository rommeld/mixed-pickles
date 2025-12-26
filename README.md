# Mixed Pickles

Simple git commit analyzer written in Rust.

A fast git commit message analyzer written in Rust, distributed as a Python package.
Checks your commit messages for common quality issues before you push.
Installation
Using uv (recommended)
bashuv tool install commit-analyzer
This installs commit-analyzer globally without affecting your project dependencies.
Using pip
bashpip install commit-analyzer
From a wheel file
If you have a .whl file (e.g., from a colleague or internal build):
bash# With uv
uv tool install commit-analyzer --from /path/to/commit_analyzer-0.1.0-\*.whl

# With pip

pip install /path/to/commit_analyzer-0.1.0-\*.whl
CLI Usage
Basic usage
bash# Analyze the last 10 commits in the current directory
commit-analyzer

# Analyze a specific repository

commit-analyzer /path/to/repo

# Analyze more commits with a stricter threshold

commit-analyzer --limit 20 --threshold 15
Options
OptionShortDefaultDescription--limit-l10Number of commits to analyze--threshold-t10Minimum acceptable message length--help-hShow help message--version-VShow version
Exit codes

0 — No issues found
1 — Issues found (or error occurred)

These exit codes make the tool suitable for use in git hooks and CI pipelines.
Git Pre-Push Hook
Run the analyzer automatically before every push.
Setup
Create .git/hooks/pre-push in your repository:
bash#!/bin/sh
commit-analyzer --limit 20 --threshold 10
Make it executable:
bashchmod +x .git/hooks/pre-push
How it works

If all commits pass: exit 0 → push proceeds
If issues found: exit 1 → push blocked

You'll see the issues in your terminal and can fix them before retrying.
Alternative: using uvx
If you haven't installed the tool permanently, use uvx to run it on-demand:
bash#!/bin/sh
uvx commit-analyzer --limit 20 --threshold 10
Python Usage
You can also import the analyzer in Python scripts:
pythonfrom commit_analyzer import analyze_commits, Commit

# Analyze commits and get results

results = analyze_commits("/path/to/repo", limit=20, threshold=10)

for commit in results:
print(f"{commit.hash[:7]} - {commit.subject}")
Available functions
analyze_commits(path, limit=10, threshold=10)
Analyzes commits and returns those with issues.

path: Path to git repository (string)
limit: Number of commits to check (int)
threshold: Minimum message length (int)
Returns: List of Commit objects with issues

get_commits(path, limit=10)
Retrieves commits without filtering.

path: Path to git repository (string)
limit: Number of commits to retrieve (int)
Returns: List of Commit objects

The Commit class
pythoncommit.hash # Full commit hash (str)
commit.author_name # Author's name (str)
commit.author_email # Author's email (str)
commit.subject # Commit message subject line (str)
Rust Usage
If you're working in Rust, you can use this as a library:
toml# Cargo.toml
[dependencies]
commit_analyzer = "0.1"
rustuse commit_analyzer::{analyze_commits, AnalyzerConfig};

fn main() -> Result<(), commit_analyzer::Error> {
let config = AnalyzerConfig {
limit: 20,
threshold: 10,
};

    let commits = analyze_commits(".", &config)?;

    for commit in commits {
        println!("{}: {}", &commit.hash[..7], commit.subject);
    }

    Ok(())

}
Note: The Rust API may differ from the examples above depending on your implementation. Check the generated documentation with cargo doc --open.
Building from Source
Requires Rust and Python 3.8+.
bash# Clone the repository
git clone https://github.com/yourusername/commit-analyzer
cd commit-analyzer

# Create a virtual environment

python -m venv .venv
source .venv/bin/activate

# Install maturin

pip install maturin

# Build and install locally

maturin develop

# Or build a wheel for distribution

maturin build --release
The wheel will be in target/wheels/.
