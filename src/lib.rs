//! Mixed Pickles - Git commit analyzer with Python bindings.

mod commit;
pub mod error;
mod git;
mod output;
mod validation;

use std::path::PathBuf;

use clap::Parser;
use error::CLIError;
use git::{fetch_commits as git_fetch_commits, validate_repo_path};
use output::print_results;
use pyo3::prelude::*;
use validation::validate_commits;

pub use commit::Commit;
pub use validation::Validation;

/// CLI arguments for the commit analyzer.
#[derive(Parser, Debug)]
#[command(name = "mixed-pickles")]
#[command(about = "Analyze git commits and find those with short messages")]
pub struct GitCLI {
    /// Path to the git repository
    #[arg(long)]
    pub path: Option<PathBuf>,
    /// Maximum number of commits to analyze
    #[arg(short, long)]
    pub limit: Option<usize>,
    /// Minimum message length in characters
    #[arg(short, long, default_value_t = 30)]
    pub threshold: usize,
    /// Suppress output unless issues found
    #[arg(short, long)]
    pub quiet: bool,
}

impl GitCLI {
    /// Run the commit analyzer with these CLI arguments.
    pub fn run(&self) -> Result<(), CLIError> {
        commit_analyzer(self.path.as_ref(), self.limit, self.threshold, self.quiet)
    }
}

/// Core commit analysis logic.
pub fn commit_analyzer(
    path: Option<&PathBuf>,
    limit: Option<usize>,
    threshold: usize,
    quiet: bool,
) -> Result<(), CLIError> {
    if let Some(p) = path {
        validate_repo_path(p)?;
    }

    let commits = git_fetch_commits(path, limit)?;
    let validation_results = validate_commits(&commits, threshold);
    let analyzed_count = commits.len();

    if !quiet || !validation_results.is_empty() {
        print_results(
            &validation_results,
            commits.len(),
            analyzed_count,
            threshold,
            &path.cloned(),
        );
    }

    if validation_results.is_empty() {
        Ok(())
    } else {
        Err(CLIError::ValidationFailed(validation_results.len()))
    }
}

/// Fetch commits from a git repository.
///
/// Args:
///     path: Path to the repository (default: current directory)
///     limit: Maximum number of commits to fetch (default: all)
///
/// Returns:
///     List of Commit objects
///
/// Raises:
///     RuntimeError: If the path is invalid or git command fails
#[pyfunction]
#[pyo3(signature = (path=None, limit=None))]
fn fetch_commits(path: Option<String>, limit: Option<usize>) -> PyResult<Vec<Commit>> {
    let path_buf = path.map(PathBuf::from);
    if let Some(ref p) = path_buf {
        validate_repo_path(p)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    }
    git_fetch_commits(path_buf.as_ref(), limit)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Analyze commits and find those which do not match pre-defined features.
///
/// Args:
///     path: Path to the repository (default: current directory)
///     limit: Number of commits to analyze (default: all)
///     threshold: Minimum message length in characters (default: 30)
///     quiet: Suppress output unless issues found (default: False)
///
/// Returns:
///     None on success
///
/// Raises:
///     RuntimeError: If validation issues are found or other errors occur
#[pyfunction]
#[pyo3(signature = (path=None, limit=None, threshold=30, quiet=false))]
fn analyze_commits(
    path: Option<String>,
    limit: Option<usize>,
    threshold: usize,
    quiet: bool,
) -> PyResult<()> {
    let path_buf = path.map(PathBuf::from);
    commit_analyzer(path_buf.as_ref(), limit, threshold, quiet)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// CLI entry point for the commit analyzer.
///
/// Parses command-line arguments and runs the analysis.
/// This function is called when running `mixed-pickles` from the command line.
/// Exits with code 1 if validation issues are found, 0 otherwise.
#[pyfunction]
fn main(py: Python<'_>) {
    // Get sys.argv from Python for correct argument parsing
    let sys = py.import("sys").expect("Failed to import sys");
    let argv: Vec<String> = sys
        .getattr("argv")
        .expect("Failed to get argv")
        .extract()
        .expect("Failed to extract argv");

    let cli = match GitCLI::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(e) => {
            // clap handles --help and --version by "failing" with a special error
            e.exit();
        }
    };

    match cli.run() {
        Ok(()) => {}
        Err(CLIError::ValidationFailed(_)) => {
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[pymodule]
mod mixed_pickles {
    #[pymodule_export]
    use super::analyze_commits;
    #[pymodule_export]
    use super::commit::Commit;
    #[pymodule_export]
    use super::fetch_commits;
    #[pymodule_export]
    use super::main;
    #[pymodule_export]
    use super::validation::Validation;
}
