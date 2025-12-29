//! Validation types and logic.

use std::fmt;
use std::sync::LazyLock;

use pyo3::prelude::*;
use regex::Regex;

use crate::commit::Commit;

/// Regex for issue/ticket references like #123, GH-456, JIRA-789, etc.
static REFERENCE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(#\d+|gh-\d+|[A-Z]{2,}-\d+)").expect("Invalid reference regex")
});

/// Regex for conventional commit format: type(scope)?: description
/// Supports: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
static CONVENTIONAL_COMMIT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?:\s.+")
        .expect("Invalid conventional commit regex")
});

/// Regex for vague language patterns in commit messages.
/// Matches descriptions that are too generic like "fix bug", "update code", "change stuff".
static VAGUE_LANGUAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(fix(ed|es|ing)?|update[ds]?|change[ds]?|modify|modified|modifies|tweak(ed|s)?|adjust(ed|s)?)\s+(it|this|that|things?|stuff|code|bug|issue|error|problem)s?\b")
        .expect("Invalid vague language regex")
});

/// Validation types for commit analysis.
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation {
    /// Commit message is too short.
    ShortCommit,
    /// Commit is missing a reference (e.g., issue number like #123).
    MissingReference,
    /// Commit does not follow conventional commits format.
    InvalidFormat,
    /// Commit uses vague language without meaningful context.
    VagueLanguage,
    // WipCommit,
    // NonImperative,
}

#[pymethods]
impl Validation {
    /// Returns a human-readable description of this validation type.
    fn __str__(&self) -> &'static str {
        match self {
            Validation::ShortCommit => "Short commit message",
            Validation::MissingReference => "Missing issue reference (e.g., #123)",
            Validation::InvalidFormat => "Invalid format (expected: type: description)",
            Validation::VagueLanguage => "Vague language (e.g., 'fix bug', 'update code')",
        }
    }

    /// Returns a string representation for debugging.
    fn __repr__(&self) -> String {
        match self {
            Validation::ShortCommit => "Validation.ShortCommit".to_string(),
            Validation::MissingReference => "Validation.MissingReference".to_string(),
            Validation::InvalidFormat => "Validation.InvalidFormat".to_string(),
            Validation::VagueLanguage => "Validation.VagueLanguage".to_string(),
        }
    }
}

impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Validation::ShortCommit => write!(f, "Short commit message"),
            Validation::MissingReference => write!(f, "Missing issue reference (e.g., #123)"),
            Validation::InvalidFormat => write!(f, "Invalid format (expected: type: description)"),
            Validation::VagueLanguage => {
                write!(f, "Vague language (e.g., 'fix bug', 'update code')")
            }
        }
    }
}

/// Check if a commit message contains an issue/ticket reference.
pub fn has_reference(subject: &str) -> bool {
    REFERENCE_REGEX.is_match(subject)
}

/// Check if a commit message follows conventional commits format.
pub fn has_conventional_format(subject: &str) -> bool {
    CONVENTIONAL_COMMIT_REGEX.is_match(subject)
}

/// Check if a commit message contains vague language.
pub fn has_vague_language(subject: &str) -> bool {
    VAGUE_LANGUAGE_REGEX.is_match(subject)
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

    mod reference_validation {
        use super::*;

        #[test]
        fn matches_github_issue_reference() {
            assert!(has_reference("fix: resolve bug #123"));
            assert!(has_reference("feat: add feature (#456)"));
            assert!(has_reference("#1 initial commit"));
        }

        #[test]
        fn matches_github_pr_reference() {
            assert!(has_reference("fix: resolve bug GH-123"));
            assert!(has_reference("feat: add feature (gh-456)"));
        }

        #[test]
        fn matches_jira_style_reference() {
            assert!(has_reference("fix: resolve bug JIRA-123"));
            assert!(has_reference("feat: add feature ABC-456"));
            assert!(has_reference("PROJ-1 initial commit"));
        }

        #[test]
        fn rejects_commits_without_reference() {
            assert!(!has_reference("fix: resolve bug"));
            assert!(!has_reference("feat: add new feature"));
            assert!(!has_reference("initial commit"));
        }

        #[test]
        fn rejects_invalid_reference_formats() {
            assert!(!has_reference("fix: resolve bug #"));
            assert!(!has_reference("fix: resolve bug A-123")); // single letter prefix
        }
    }

    mod conventional_format_validation {
        use super::*;

        #[test]
        fn matches_feat_type() {
            assert!(has_conventional_format("feat: add new feature"));
            assert!(has_conventional_format("feat(api): add new endpoint"));
        }

        #[test]
        fn matches_fix_type() {
            assert!(has_conventional_format("fix: resolve bug"));
            assert!(has_conventional_format("fix(ui): fix button color"));
        }

        #[test]
        fn matches_other_types() {
            assert!(has_conventional_format("docs: update README"));
            assert!(has_conventional_format("style: format code"));
            assert!(has_conventional_format("refactor: simplify logic"));
            assert!(has_conventional_format("perf: improve speed"));
            assert!(has_conventional_format("test: add unit tests"));
            assert!(has_conventional_format("build: update dependencies"));
            assert!(has_conventional_format("ci: fix pipeline"));
            assert!(has_conventional_format("chore: update config"));
            assert!(has_conventional_format("revert: undo changes"));
        }

        #[test]
        fn matches_breaking_change_indicator() {
            assert!(has_conventional_format("feat!: breaking change"));
            assert!(has_conventional_format("fix(api)!: breaking fix"));
        }

        #[test]
        fn matches_scope_with_special_chars() {
            assert!(has_conventional_format("feat(api-v2): add endpoint"));
            assert!(has_conventional_format("fix(user/auth): fix login"));
        }

        #[test]
        fn rejects_missing_type() {
            assert!(!has_conventional_format("add new feature"));
            assert!(!has_conventional_format(": add new feature"));
        }

        #[test]
        fn rejects_missing_colon() {
            assert!(!has_conventional_format("feat add new feature"));
            assert!(!has_conventional_format("fix resolve bug"));
        }

        #[test]
        fn rejects_missing_space_after_colon() {
            assert!(!has_conventional_format("feat:add new feature"));
            assert!(!has_conventional_format("fix:resolve bug"));
        }

        #[test]
        fn rejects_unknown_types() {
            assert!(!has_conventional_format("feature: add new feature"));
            assert!(!has_conventional_format("bugfix: resolve bug"));
            assert!(!has_conventional_format("update: change something"));
        }

        #[test]
        fn rejects_empty_description() {
            assert!(!has_conventional_format("feat: "));
            assert!(!has_conventional_format("fix:"));
        }
    }

    mod vague_language_validation {
        use super::*;

        #[test]
        fn detects_fix_bug() {
            assert!(has_vague_language("fix bug"));
            assert!(has_vague_language("fixed bug"));
            assert!(has_vague_language("fixes bug"));
            assert!(has_vague_language("fixing bug"));
        }

        #[test]
        fn detects_update_code() {
            assert!(has_vague_language("update code"));
            assert!(has_vague_language("updated code"));
            assert!(has_vague_language("updates code"));
        }

        #[test]
        fn detects_change_stuff() {
            assert!(has_vague_language("change stuff"));
            assert!(has_vague_language("changed things"));
            assert!(has_vague_language("changes it"));
        }

        #[test]
        fn detects_modify_patterns() {
            assert!(has_vague_language("modify code"));
            assert!(has_vague_language("modified this"));
            assert!(has_vague_language("modifies that"));
        }

        #[test]
        fn detects_tweak_adjust_patterns() {
            assert!(has_vague_language("tweak code"));
            assert!(has_vague_language("tweaked stuff"));
            assert!(has_vague_language("adjust things"));
            assert!(has_vague_language("adjusted it"));
        }

        #[test]
        fn detects_issue_error_problem() {
            assert!(has_vague_language("fix issue"));
            assert!(has_vague_language("fix error"));
            assert!(has_vague_language("fix problem"));
            assert!(has_vague_language("fixed issues"));
        }

        #[test]
        fn allows_specific_descriptions() {
            assert!(!has_vague_language(
                "fix: resolve null pointer in user login"
            ));
            assert!(!has_vague_language("fix: handle edge case in parser"));
            assert!(!has_vague_language("update README with installation steps"));
            assert!(!has_vague_language("change default timeout to 30 seconds"));
        }

        #[test]
        fn allows_conventional_commits_with_context() {
            assert!(!has_vague_language(
                "feat: add user authentication with OAuth2"
            ));
            assert!(!has_vague_language(
                "fix: resolve memory leak in connection pool"
            ));
            assert!(!has_vague_language("docs: update API documentation"));
        }

        #[test]
        fn case_insensitive() {
            assert!(has_vague_language("FIX BUG"));
            assert!(has_vague_language("Fix Bug"));
            assert!(has_vague_language("UPDATE CODE"));
        }
    }

    mod validate_commits_tests {
        use super::*;

        fn create_valid_commit(subject: &str) -> Commit {
            Commit {
                hash: "1111111111111111111111111111111111111111".to_string(),
                author_name: "Author".to_string(),
                author_email: "a@b.com".to_string(),
                subject: subject.to_string(),
            }
        }

        #[test]
        fn valid_commit_passes_all_validations() {
            let commits = vec![create_valid_commit(
                "feat: add new feature for user authentication #123",
            )];
            let results = validate_commits(&commits, 10);
            assert!(results.is_empty(), "Expected no failures for valid commit");
        }

        #[test]
        fn commit_missing_reference_fails() {
            let commits = vec![create_valid_commit("feat: add new feature")];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::MissingReference));
        }

        #[test]
        fn commit_with_invalid_format_fails() {
            let commits = vec![create_valid_commit("add new feature #123")];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::InvalidFormat));
        }

        #[test]
        fn short_commit_fails() {
            let commits = vec![create_valid_commit("feat: x #1")];
            let results = validate_commits(&commits, 20);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::ShortCommit));
        }

        #[test]
        fn commit_can_have_multiple_failures() {
            let commits = vec![create_valid_commit("bad")];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::ShortCommit));
            assert!(results[0].failures.contains(&Validation::MissingReference));
            assert!(results[0].failures.contains(&Validation::InvalidFormat));
        }

        #[test]
        fn commit_with_vague_language_fails() {
            let commits = vec![create_valid_commit("feat: fix bug #123")];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::VagueLanguage));
        }
    }
}
