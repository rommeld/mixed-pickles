mod commit;
pub mod error;

use std::path::PathBuf;

use commit::{Commit, print_results, validate_repo_path};
use error::CLIError;
use pyo3::prelude::*;

pub fn commit_analyzer(
    path: Option<&PathBuf>,
    limit: Option<usize>,
    threshold: usize,
    quiet: bool,
) -> Result<(), CLIError> {
    if let Some(p) = path {
        validate_repo_path(p)?;
    }

    let commits = Commit::fetch_all(path, limit)?;
    let short_commits = Commit::find_short(&commits, threshold);
    let analyzed_count = commits.len();

    if !quiet || !short_commits.is_empty() {
        print_results(
            &short_commits,
            commits.len(),
            analyzed_count,
            threshold,
            &path.cloned(),
        );
    }

    if short_commits.is_empty() {
        Ok(())
    } else {
        Err(CLIError::ShortCommitsFound(short_commits.len()))
    }
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
///     RuntimeError: If short commits are found or other errors occur
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
/// Exits with code 1 if short commits are found, 0 otherwise.
#[pyfunction]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut path: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut threshold: usize = 30;
    let mut quiet: bool = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("Usage: mixed-pickles [OPTIONS]");
                println!();
                println!("Analyze git commits and find those with short messages.");
                println!();
                println!("Options:");
                println!(
                    "  --path <PATH>        Path to the git repository (default: current directory)"
                );
                println!(
                    "  -l, --limit <N>      Maximum number of commits to analyze (default: all)"
                );
                println!(
                    "  -t, --threshold <N>  Minimum message length in characters (default: 30)"
                );
                println!("  -q, --quiet          Suppress output unless issues found");
                println!("  -h, --help           Show this help message");
                return;
            }
            "--path" => {
                i += 1;
                if i < args.len() {
                    path = Some(args[i].clone());
                }
            }
            "-l" | "--limit" => {
                i += 1;
                if i < args.len() {
                    limit = args[i].parse().ok();
                }
            }
            "-t" | "--threshold" => {
                i += 1;
                if i < args.len() {
                    threshold = args[i].parse().unwrap_or(30);
                }
            }
            "-q" | "--quiet" => {
                quiet = true;
            }
            _ => {}
        }
        i += 1;
    }

    let path_buf = path.map(PathBuf::from);
    match commit_analyzer(path_buf.as_ref(), limit, threshold, quiet) {
        Ok(()) => {}
        Err(CLIError::ShortCommitsFound(_)) => {
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
    use super::main;
}
