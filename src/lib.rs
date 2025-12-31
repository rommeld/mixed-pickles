//! Mixed Pickles - Git commit analyzer.

mod commit;
mod config;
pub mod error;
mod git;
mod output;
mod validation;

use std::path::{Path, PathBuf};

use clap::Parser;
use config::{ConfigFile, find_config_file, load_config};
use error::CLIError;
use git::{count_commits, fetch_commits as git_fetch_commits, validate_repo_path};
use output::print_results;
use pyo3::prelude::*;
use validation::{Severity, ValidationConfig, validate_commits};

pub use commit::Commit;
pub use validation::Validation;

#[derive(Parser, Debug)]
#[command(name = "mixed-pickles")]
#[command(about = "Analyze git commits and find those with short messages")]
#[command(after_help = "CONFIGURATION:
    Settings can be specified in pyproject.toml under [tool.mixed-pickles]
    or in a dedicated .mixed-pickles.toml file.

    Example pyproject.toml:
        [tool.mixed-pickles]
        threshold = 50
        disable = [\"format\", \"reference\"]

        [tool.mixed-pickles.severity]
        short = \"error\"
        wip = \"warning\"

    CLI arguments override configuration file settings.")]
pub struct GitCLI {
    #[arg(long)]
    pub path: Option<PathBuf>,
    #[arg(short, long)]
    pub limit: Option<usize>,
    #[arg(short, long)]
    pub threshold: Option<usize>,
    #[arg(short, long)]
    pub quiet: bool,
    #[arg(long, value_name = "VALIDATIONS")]
    pub error: Option<String>,
    #[arg(long, value_name = "VALIDATIONS")]
    pub warn: Option<String>,
    #[arg(long, value_name = "VALIDATIONS")]
    pub ignore: Option<String>,
    #[arg(long, value_name = "VALIDATIONS")]
    pub disable: Option<String>,
    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,
    /// Path to configuration file (default: auto-detect pyproject.toml)
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Ignore configuration file
    #[arg(long)]
    pub no_config: bool,
}

impl GitCLI {
    /// Apply CLI argument overrides to an existing config.
    fn apply_cli_overrides(&self, config: &mut ValidationConfig) -> Result<(), CLIError> {
        if let Some(threshold) = self.threshold {
            config.threshold = threshold;
        }
        if let Some(ref disables) = self.disable {
            config
                .parse_and_disable(disables)
                .map_err(CLIError::InvalidValidation)?;
        }
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
        Ok(())
    }

    /// Find config file: use --config if provided, otherwise auto-detect.
    fn find_config(&self) -> Option<ConfigFile> {
        if let Some(ref path) = self.config {
            // User specified explicit config path
            if path.exists() {
                // Determine type based on filename
                if path.file_name().is_some_and(|n| n == "pyproject.toml") {
                    return Some(ConfigFile::PyProjectToml(path.clone()));
                } else {
                    return Some(ConfigFile::Dedicated(path.clone()));
                }
            }
            return None;
        }

        // Auto-detect from repo path or current directory
        let start = self.path.as_deref().unwrap_or(Path::new("."));
        find_config_file(start)
    }

    pub fn run(&self) -> Result<(), CLIError> {
        let mut config = ValidationConfig::default();

        // Load config file unless disabled
        if !self.no_config
            && let Some(config_file) = self.find_config()
        {
            let file_config = load_config(&config_file)?;
            config.apply_file_config(&file_config)?;
        }

        // Apply CLI overrides (takes precedence over file config)
        self.apply_cli_overrides(&mut config)?;

        commit_analyzer(
            self.path.as_ref(),
            self.limit,
            self.quiet,
            self.strict,
            &config,
        )
    }
}

pub fn commit_analyzer(
    path: Option<&PathBuf>,
    limit: Option<usize>,
    quiet: bool,
    strict: bool,
    config: &ValidationConfig,
) -> Result<(), CLIError> {
    if let Some(p) = path {
        validate_repo_path(p)?;
    }

    let total_commits = count_commits(path)?;
    let commits = git_fetch_commits(path, limit)?;

    let validation_results = validate_commits(&commits, config);
    let analyzed_count = commits.len();

    let has_errors = validation_results.iter().any(|r| r.has_errors());
    let has_warnings = validation_results.iter().any(|r| r.has_warnings());

    if !quiet || !validation_results.is_empty() {
        print_results(
            &validation_results,
            total_commits,
            analyzed_count,
            config.threshold,
            &path.cloned(),
        );
    }

    if has_errors || (strict && has_warnings) {
        let error_count = validation_results.iter().filter(|r| r.has_errors()).count();
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
///     quiet: Suppress output unless issues found (default: False)
///     strict: Treat warnings as errors (default: False)
///     config: ValidationConfig object for customizing validation behavior.
///             If None and use_config=True, auto-discovers pyproject.toml.
///     use_config: Whether to auto-load configuration from pyproject.toml (default: True).
///                 Ignored if config is provided.
///
/// Returns:
///     None on success
///
/// Raises:
///     RuntimeError: If validation issues are found or other errors occur
#[pyfunction]
#[pyo3(signature = (path=None, limit=None, quiet=false, strict=false, config=None, use_config=true))]
fn analyze_commits(
    path: Option<String>,
    limit: Option<usize>,
    quiet: bool,
    strict: bool,
    config: Option<ValidationConfig>,
    use_config: bool,
) -> PyResult<()> {
    let path_buf = path.map(PathBuf::from);

    let validation_config = if let Some(cfg) = config {
        cfg
    } else if use_config {
        // Auto-discover and load config
        let start_dir = path_buf.as_deref().unwrap_or(Path::new("."));

        let mut cfg = ValidationConfig::default();
        if let Some(config_file) = find_config_file(start_dir) {
            let file_config = load_config(&config_file)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            cfg.apply_file_config(&file_config)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }
        cfg
    } else {
        ValidationConfig::default()
    };

    commit_analyzer(path_buf.as_ref(), limit, quiet, strict, &validation_config)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// CLI entry point. Exits with code 1 if validation issues are found.
#[pyfunction]
fn main(py: Python<'_>) {
    let sys = py.import("sys").expect("Failed to import sys");
    let argv: Vec<String> = sys
        .getattr("argv")
        .expect("Failed to get argv")
        .extract()
        .expect("Failed to extract argv");

    let cli = match GitCLI::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    match cli.run() {
        Ok(()) => {}
        Err(CLIError::ValidationFailed(_)) => std::process::exit(1),
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
    #[pymodule_export]
    use super::validation::ValidationConfig;
}
