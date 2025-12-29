//! Validation types and logic.

use std::fmt;

use pyo3::prelude::*;

use crate::commit::Commit;

/// Validation types for commit analysis.
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation {
    /// Commit message is too short.
    ShortCommit,
    /// Commit is missing a reference (e.g., commit pre-fix, issue number).
    MissingReference,
    // VagueLanguage,
    // CommitFormat,
    // WipCommit,
    // NonImperative,
}

#[pymethods]
impl Validation {
    /// Returns a human-readable description of this validation type.
    fn __str__(&self) -> &'static str {
        match self {
            Validation::ShortCommit => "Short commit message",
            Validation::MissingReference => "Missing reference",
        }
    }

    /// Returns a string representation for debugging.
    fn __repr__(&self) -> String {
        match self {
            Validation::ShortCommit => "Validation.ShortCommit".to_string(),
            Validation::MissingReference => "Validation.MissingReference".to_string(),
        }
    }
}

impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Validation::ShortCommit => write!(f, "Short commit message"),
            Validation::MissingReference => write!(f, "Missing reference"),
        }
    }
}

/// A commit paired with its validation failures.
#[derive(Debug)]
pub struct ValidationResult<'a> {
    pub commit: &'a Commit,
    pub failures: Vec<Validation>,
}

/// Validate all commits and return only those with failures.
pub fn validate_commits(commits: &[Commit], threshold: usize) -> Vec<ValidationResult<'_>> {
    commits
        .iter()
        .map(|commit| ValidationResult {
            commit,
            failures: commit.validate_internal(threshold),
        })
        .filter(|result| !result.failures.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commits() -> Vec<Commit> {
        vec![
            Commit {
                hash: "1111111111111111111111111111111111111111".to_string(),
                author_name: "Author".to_string(),
                author_email: "a@b.com".to_string(),
                subject: "short".to_string(),
            },
            Commit {
                hash: "2222222222222222222222222222222222222222".to_string(),
                author_name: "Author".to_string(),
                author_email: "a@b.com".to_string(),
                subject: "this is a much longer commit message".to_string(),
            },
            Commit {
                hash: "3333333333333333333333333333333333333333".to_string(),
                author_name: "Author".to_string(),
                author_email: "a@b.com".to_string(),
                subject: "tiny".to_string(),
            },
        ]
    }

    #[test]
    fn validate_commits_finds_short_commits() {
        let commits = create_test_commits();
        let results = validate_commits(&commits, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].commit.subject(), "short");
        assert_eq!(results[1].commit.subject(), "tiny");
    }

    #[test]
    fn validate_commits_returns_empty_when_no_issues() {
        let commits = create_test_commits();
        let results = validate_commits(&commits, 3);
        assert!(results.is_empty());
    }

    #[test]
    fn validate_commits_returns_empty_for_empty_list() {
        let commits: Vec<Commit> = vec![];
        let results = validate_commits(&commits, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn validation_result_contains_correct_failures() {
        let commits = create_test_commits();
        let results = validate_commits(&commits, 10);
        assert!(!results.is_empty());
        assert!(results[0].failures.contains(&Validation::ShortCommit));
    }
}
