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
) -> Result<(), CLIError> {
    if let Some(p) = path {
        validate_repo_path(p)?;
    }

    let commits = Commit::fetch_all(path, limit)?;
    let short_commits = Commit::find_short(&commits, threshold);
    let analyzed_count = commits.len();

    print_results(
        &short_commits,
        commits.len(),
        analyzed_count,
        threshold,
        &path.cloned(),
    );

    Ok(())
}

/// Analyze commits and find those which do not match pre-defined features
///
/// Args:
///     path(str): Path to the repository (default: current directory)
///     limit(int): Number of commits to analyze (default: all)
///     threshold(int): Number of characters in a commit message (default: 30)
#[pyfunction]
#[pyo3(signature = (path=None, limit=None, threshold=30))]
fn analyze_commits(path: Option<String>, limit: Option<usize>, threshold: usize) -> PyResult<()> {
    let path_buf = path.map(PathBuf::from);
    commit_analyzer(path_buf.as_ref(), limit, threshold)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

#[pymodule]
mod mixed_pickles {
    #[pymodule_export]
    use super::analyze_commits;
}
