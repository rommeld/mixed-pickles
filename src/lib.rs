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
use validation::{Severity, ValidationConfig, validate_commits};

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
    /// Validations to treat as errors (comma-separated)
    /// Available: ShortCommit, MissingReference, InvalidFormat, VagueLanguage, WipCommit, NonImperative
    /// Aliases: short, ref, format, vague, wip, imperative
    #[arg(long, value_name = "VALIDATIONS")]
    pub error: Option<String>,
    /// Validations to treat as warnings (comma-separated)
    #[arg(long, value_name = "VALIDATIONS")]
    pub warn: Option<String>,
    /// Validations to ignore (comma-separated)
    #[arg(long, value_name = "VALIDATIONS")]
    pub ignore: Option<String>,
}

impl GitCLI {
    /// Build a ValidationConfig from CLI arguments.
    pub fn build_config(&self) -> Result<ValidationConfig, CLIError> {
        let mut config = ValidationConfig::new();

        if let Some(ref errors) = self.error {
            config
                .parse_and_set(errors, Severity::Error)
                .map_err(CLIError::InvalidValidation)?;
        }
        if let Some(ref warnings) = self.warn {
            config
                .parse_and_set(warnings, Severity::Warning)
                .map_err(CLIError::InvalidValidation)?;
        }
        if let Some(ref ignores) = self.ignore {
            config
                .parse_and_set(ignores, Severity::Ignore)
                .map_err(CLIError::InvalidValidation)?;
        }

        Ok(config)
    }

    /// Run the commit analyzer with these CLI arguments.
    pub fn run(&self) -> Result<(), CLIError> {
        let config = self.build_config()?;
        commit_analyzer(
            self.path.as_ref(),
            self.limit,
            self.threshold,
            self.quiet,
            &config,
        )
    }
}

/// Core commit analysis logic.
pub fn commit_analyzer(
    path: Option<&PathBuf>,
    limit: Option<usize>,
    threshold: usize,
    quiet: bool,
    config: &ValidationConfig,
) -> Result<(), CLIError> {
    if let Some(p) = path {
        validate_repo_path(p)?;
    }

    let commits = git_fetch_commits(path, limit)?;
    let validation_results = validate_commits(&commits, threshold);
    let analyzed_count = commits.len();

    // Filter results to only include reported validations
    let filtered_results: Vec<_> = validation_results
        .into_iter()
        .map(|mut result| {
            result.failures.retain(|v| config.should_report(v));
            result
        })
        .filter(|result| !result.failures.is_empty())
        .collect();

    // Check if any errors exist
    let has_errors = filtered_results
        .iter()
        .any(|r| r.failures.iter().any(|v| config.is_error(v)));

    if !quiet || !filtered_results.is_empty() {
        print_results(
            &filtered_results,
            commits.len(),
            analyzed_count,
            threshold,
            &path.cloned(),
            config,
        );
    }

    if has_errors {
        let error_count = filtered_results
            .iter()
            .filter(|r| r.failures.iter().any(|v| config.is_error(v)))
            .count();
        Err(CLIError::ValidationFailed(error_count))
    } else {
        Ok(())
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
///     errors: Comma-separated validation types to treat as errors
///     warnings: Comma-separated validation types to treat as warnings
///     ignore: Comma-separated validation types to ignore
///
/// Returns:
///     None on success
///
/// Raises:
///     RuntimeError: If validation issues are found or other errors occur
#[pyfunction]
#[pyo3(signature = (path=None, limit=None, threshold=30, quiet=false, errors=None, warnings=None, ignore=None))]
fn analyze_commits(
    path: Option<String>,
    limit: Option<usize>,
    threshold: usize,
    quiet: bool,
    errors: Option<String>,
    warnings: Option<String>,
    ignore: Option<String>,
) -> PyResult<()> {
    let path_buf = path.map(PathBuf::from);

    let mut config = ValidationConfig::new();
    if let Some(ref e) = errors {
        config
            .parse_and_set(e, Severity::Error)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    }
    if let Some(ref w) = warnings {
        config
            .parse_and_set(w, Severity::Warning)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    }
    if let Some(ref i) = ignore {
        config
            .parse_and_set(i, Severity::Ignore)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
    }

    commit_analyzer(path_buf.as_ref(), limit, threshold, quiet, &config)
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
    use super::validation::Severity;
    #[pymodule_export]
    use super::validation::Validation;
}
