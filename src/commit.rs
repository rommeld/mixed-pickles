//! Commit struct.

use pyo3::prelude::*;

use crate::validation::{Validation, ValidationConfig, validate_commit};

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
    #[getter]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    #[getter]
    pub fn author_name(&self) -> &str {
        &self.author_name
    }

    #[getter]
    pub fn author_email(&self) -> &str {
        &self.author_email
    }

    #[getter]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn is_short(&self, threshold: usize) -> bool {
        self.subject.len() <= threshold
    }

    /// Validate this commit against all rules.
    ///
    /// Returns:
    ///     List of Validation types that failed.
    #[pyo3(signature = (threshold=30))]
    pub fn validate(&self, threshold: usize) -> Vec<Validation> {
        let mut config = ValidationConfig::default();
        config.threshold = threshold;
        validate_commit(self, &config)
            .into_iter()
            .map(|f| f.validation)
            .collect()
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
