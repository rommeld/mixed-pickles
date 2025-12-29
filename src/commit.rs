//! Commit struct and Python bindings.

use pyo3::prelude::*;

use crate::validation::{
    Validation, has_conventional_format, has_reference, has_vague_language, is_wip_commit,
};

/// A git commit with its metadata.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Commit {
    pub(crate) hash: String,
    pub(crate) author_name: String,
    pub(crate) author_email: String,
    pub(crate) subject: String,
}

#[pymethods]
impl Commit {
    /// The commit hash.
    #[getter]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// The commit author's name.
    #[getter]
    pub fn author_name(&self) -> &str {
        &self.author_name
    }

    /// The commit author's email address.
    #[getter]
    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    /// The commit message subject line.
    #[getter]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Check if this commit's subject is shorter than the threshold.
    pub fn is_short(&self, threshold: usize) -> bool {
        self.subject.len() <= threshold
    }

    /// Validate this commit against all validation rules.
    ///
    /// Args:
    ///     threshold: Minimum message length in characters (default: 30)
    ///
    /// Returns:
    ///     List of Validation types that failed for this commit.
    #[pyo3(signature = (threshold=30))]
    pub fn validate(&self, threshold: usize) -> Vec<Validation> {
        self.validate_internal(threshold)
    }
}

impl Commit {
    /// Internal validation logic.
    pub(crate) fn validate_internal(&self, threshold: usize) -> Vec<Validation> {
        let mut failures = Vec::new();

        if self.is_short(threshold) {
            failures.push(Validation::ShortCommit);
        }

        if !has_reference(&self.subject) {
            failures.push(Validation::MissingReference);
        }

        if !has_conventional_format(&self.subject) {
            failures.push(Validation::InvalidFormat);
        }

        if has_vague_language(&self.subject) {
            failures.push(Validation::VagueLanguage);
        }

        if is_wip_commit(&self.subject) {
            failures.push(Validation::WipCommit);
        }

        failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commit(hash: &str, subject: &str) -> Commit {
        Commit {
            hash: hash.to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            subject: subject.to_string(),
        }
    }

    #[test]
    fn is_short_returns_true_for_short_subject() {
        let commit = create_test_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "fix bug");
        assert!(commit.is_short(10));
    }

    #[test]
    fn is_short_returns_false_for_long_subject() {
        let commit = create_test_commit(
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
            "feat: implement user authentication with OAuth2",
        );
        assert!(!commit.is_short(10));
    }

    #[test]
    fn is_short_returns_true_when_equal_to_threshold() {
        let commit = create_test_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "1234567890");
        assert!(commit.is_short(10));
    }
}
