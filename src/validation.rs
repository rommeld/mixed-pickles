//! Validation types and logic.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
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

/// Regex for WIP (work-in-progress) commit patterns.
/// Matches commits that shouldn't be in final history like "WIP", "fixup!", "squash!".
static WIP_COMMIT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(^wip\b|^wip:|^\[wip\]|\bwork.?in.?progress\b|^fixup!|^squash!|^amend!|\bdo\s*not\s*merge\b|\bdon'?t\s*merge\b|\bwip\s*$)")
        .expect("Invalid WIP commit regex")
});

/// Regex for non-imperative mood patterns in commit messages.
/// Detects past tense (-ed) and present continuous (-ing) verb forms at the start.
static NON_IMPERATIVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Match after optional conventional commit prefix (type(scope): )
    // Common past tense and -ing forms that indicate non-imperative mood
    Regex::new(r"(?i)^(?:(?:feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(?:\([^)]+\))?!?:\s*)?(added|removed|fixed|updated|changed|implemented|created|deleted|modified|refactored|improved|resolved|merged|moved|renamed|replaced|cleaned|enabled|disabled|converted|introduced|integrated|adjusted|corrected|enhanced|extended|optimized|simplified|upgraded|migrated|adding|removing|fixing|updating|changing|implementing|creating|deleting|modifying|refactoring|improving|resolving|merging|moving|renaming|replacing|cleaning|enabling|disabling|converting|introducing|integrating|adjusting|correcting|enhancing|extending|optimizing|simplifying|upgrading|migrating)\b")
        .expect("Invalid non-imperative regex")
});

/// Validation types for commit analysis.
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Validation {
    /// Commit message is too short.
    ShortCommit,
    /// Commit is missing a reference (e.g., issue number like #123).
    MissingReference,
    /// Commit does not follow conventional commits format.
    InvalidFormat,
    /// Commit uses vague language without meaningful context.
    VagueLanguage,
    /// Commit is a work-in-progress and shouldn't be in final history.
    WipCommit,
    /// Commit message doesn't use imperative mood.
    NonImperative,
}

/// All validation types for iteration.
pub const ALL_VALIDATIONS: [Validation; 6] = [
    Validation::ShortCommit,
    Validation::MissingReference,
    Validation::InvalidFormat,
    Validation::VagueLanguage,
    Validation::WipCommit,
    Validation::NonImperative,
];

impl Validation {
    /// Returns the validation name as a string (for CLI parsing).
    pub fn name(&self) -> &'static str {
        match self {
            Validation::ShortCommit => "ShortCommit",
            Validation::MissingReference => "MissingReference",
            Validation::InvalidFormat => "InvalidFormat",
            Validation::VagueLanguage => "VagueLanguage",
            Validation::WipCommit => "WipCommit",
            Validation::NonImperative => "NonImperative",
        }
    }
}

impl FromStr for Validation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shortcommit" | "short" => Ok(Validation::ShortCommit),
            "missingreference" | "reference" | "ref" => Ok(Validation::MissingReference),
            "invalidformat" | "format" => Ok(Validation::InvalidFormat),
            "vaguelanguage" | "vague" => Ok(Validation::VagueLanguage),
            "wipcommit" | "wip" => Ok(Validation::WipCommit),
            "nonimperative" | "imperative" => Ok(Validation::NonImperative),
            _ => Err(format!("Unknown validation type: {}", s)),
        }
    }
}

/// Severity level for validation findings.
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Finding blocks the operation (non-zero exit code).
    Error,
    /// Finding is reported but doesn't block.
    Warning,
    /// Informational finding.
    Info,
    /// Finding is not reported.
    Ignore,
}

#[pymethods]
impl Severity {
    fn __str__(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Ignore => "ignore",
        }
    }

    fn __repr__(&self) -> String {
        match self {
            Severity::Error => "Severity.Error".to_string(),
            Severity::Warning => "Severity.Warning".to_string(),
            Severity::Info => "Severity.Info".to_string(),
            Severity::Ignore => "Severity.Ignore".to_string(),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Ignore => write!(f, "ignore"),
        }
    }
}

/// Configuration for validation severity levels.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    severities: HashMap<Validation, Severity>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut severities = HashMap::new();
        // Default severities - sensible defaults that can be overridden
        severities.insert(Validation::WipCommit, Severity::Error);
        severities.insert(Validation::ShortCommit, Severity::Warning);
        severities.insert(Validation::VagueLanguage, Severity::Warning);
        severities.insert(Validation::NonImperative, Severity::Warning);
        severities.insert(Validation::MissingReference, Severity::Info);
        severities.insert(Validation::InvalidFormat, Severity::Info);
        Self { severities }
    }
}

impl ValidationConfig {
    /// Create a new configuration with default severities.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the severity for a specific validation type.
    pub fn set_severity(&mut self, validation: Validation, severity: Severity) {
        self.severities.insert(validation, severity);
    }

    /// Get the severity for a validation type.
    pub fn get_severity(&self, validation: &Validation) -> Severity {
        self.severities
            .get(validation)
            .copied()
            .unwrap_or(Severity::Warning)
    }

    /// Check if a validation should be reported (not ignored).
    pub fn should_report(&self, validation: &Validation) -> bool {
        self.get_severity(validation) != Severity::Ignore
    }

    /// Check if a validation is an error.
    pub fn is_error(&self, validation: &Validation) -> bool {
        self.get_severity(validation) == Severity::Error
    }

    /// Parse a comma-separated list of validation names and set their severity.
    pub fn parse_and_set(&mut self, validations: &str, severity: Severity) -> Result<(), String> {
        for name in validations.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let validation = Validation::from_str(name)?;
            self.set_severity(validation, severity);
        }
        Ok(())
    }
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
            Validation::WipCommit => "Work-in-progress commit (e.g., 'WIP', 'fixup!')",
            Validation::NonImperative => "Non-imperative mood (use 'Add' not 'Added')",
        }
    }

    /// Returns a string representation for debugging.
    fn __repr__(&self) -> String {
        match self {
            Validation::ShortCommit => "Validation.ShortCommit".to_string(),
            Validation::MissingReference => "Validation.MissingReference".to_string(),
            Validation::InvalidFormat => "Validation.InvalidFormat".to_string(),
            Validation::VagueLanguage => "Validation.VagueLanguage".to_string(),
            Validation::WipCommit => "Validation.WipCommit".to_string(),
            Validation::NonImperative => "Validation.NonImperative".to_string(),
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
            Validation::WipCommit => {
                write!(f, "Work-in-progress commit (e.g., 'WIP', 'fixup!')")
            }
            Validation::NonImperative => {
                write!(f, "Non-imperative mood (use 'Add' not 'Added')")
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

/// Check if a commit message indicates work-in-progress.
pub fn is_wip_commit(subject: &str) -> bool {
    WIP_COMMIT_REGEX.is_match(subject)
}

/// Check if a commit message uses non-imperative mood.
pub fn is_non_imperative(subject: &str) -> bool {
    NON_IMPERATIVE_REGEX.is_match(subject)
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

    mod wip_commit_validation {
        use super::*;

        #[test]
        fn detects_wip_prefix() {
            assert!(is_wip_commit("WIP add new feature"));
            assert!(is_wip_commit("wip add new feature"));
            assert!(is_wip_commit("WIP: add new feature"));
            assert!(is_wip_commit("wip: add new feature"));
        }

        #[test]
        fn detects_wip_in_brackets() {
            assert!(is_wip_commit("[WIP] add new feature"));
            assert!(is_wip_commit("[wip] add new feature"));
        }

        #[test]
        fn detects_wip_suffix() {
            assert!(is_wip_commit("add new feature WIP"));
            assert!(is_wip_commit("add new feature wip"));
        }

        #[test]
        fn detects_work_in_progress() {
            assert!(is_wip_commit("work in progress"));
            assert!(is_wip_commit("Work In Progress"));
            assert!(is_wip_commit("work-in-progress"));
            assert!(is_wip_commit("feat: add feature (work in progress)"));
        }

        #[test]
        fn detects_fixup_commits() {
            assert!(is_wip_commit("fixup! feat: add new feature"));
            assert!(is_wip_commit("fixup! fix typo"));
        }

        #[test]
        fn detects_squash_commits() {
            assert!(is_wip_commit("squash! feat: add new feature"));
            assert!(is_wip_commit("squash! fix typo"));
        }

        #[test]
        fn detects_amend_commits() {
            assert!(is_wip_commit("amend! feat: add new feature"));
            assert!(is_wip_commit("amend! fix typo"));
        }

        #[test]
        fn detects_do_not_merge() {
            assert!(is_wip_commit("feat: add feature - DO NOT MERGE"));
            assert!(is_wip_commit("do not merge"));
            assert!(is_wip_commit("don't merge"));
            assert!(is_wip_commit("dont merge"));
        }

        #[test]
        fn allows_normal_commits() {
            assert!(!is_wip_commit("feat: add new feature #123"));
            assert!(!is_wip_commit("fix: resolve memory leak"));
            assert!(!is_wip_commit("docs: update README"));
        }

        #[test]
        fn allows_wip_in_middle_of_word() {
            // "wip" in the middle of a word should not trigger
            assert!(!is_wip_commit("feat: add wiping functionality"));
            assert!(!is_wip_commit("fix: handle equipped items"));
        }
    }

    mod non_imperative_validation {
        use super::*;

        #[test]
        fn detects_past_tense_added() {
            assert!(is_non_imperative("Added new feature"));
            assert!(is_non_imperative("added user authentication"));
            assert!(is_non_imperative("feat: Added new endpoint"));
        }

        #[test]
        fn detects_past_tense_fixed() {
            assert!(is_non_imperative("Fixed bug in parser"));
            assert!(is_non_imperative("fix: Fixed memory leak"));
        }

        #[test]
        fn detects_past_tense_updated() {
            assert!(is_non_imperative("Updated dependencies"));
            assert!(is_non_imperative("docs: Updated README"));
        }

        #[test]
        fn detects_past_tense_removed() {
            assert!(is_non_imperative("Removed unused code"));
            assert!(is_non_imperative("refactor: Removed dead code"));
        }

        #[test]
        fn detects_past_tense_other_verbs() {
            assert!(is_non_imperative("Changed configuration"));
            assert!(is_non_imperative("Implemented feature"));
            assert!(is_non_imperative("Created new module"));
            assert!(is_non_imperative("Deleted old files"));
            assert!(is_non_imperative("Modified settings"));
            assert!(is_non_imperative("Refactored code"));
            assert!(is_non_imperative("Improved performance"));
            assert!(is_non_imperative("Resolved conflict"));
            assert!(is_non_imperative("Merged branch"));
            assert!(is_non_imperative("Moved files"));
            assert!(is_non_imperative("Renamed variable"));
        }

        #[test]
        fn detects_present_continuous() {
            assert!(is_non_imperative("Adding new feature"));
            assert!(is_non_imperative("Fixing bug"));
            assert!(is_non_imperative("Updating tests"));
            assert!(is_non_imperative("Removing unused imports"));
            assert!(is_non_imperative("feat: Implementing auth"));
        }

        #[test]
        fn allows_imperative_mood() {
            assert!(!is_non_imperative("Add new feature"));
            assert!(!is_non_imperative("Fix bug in parser"));
            assert!(!is_non_imperative("Update dependencies"));
            assert!(!is_non_imperative("Remove unused code"));
            assert!(!is_non_imperative("feat: Add user authentication"));
            assert!(!is_non_imperative("fix: Resolve memory leak"));
        }

        #[test]
        fn allows_imperative_with_conventional_prefix() {
            assert!(!is_non_imperative("feat: Add new endpoint"));
            assert!(!is_non_imperative("fix(api): Handle edge case"));
            assert!(!is_non_imperative("docs: Update README"));
            assert!(!is_non_imperative("refactor!: Simplify logic"));
        }

        #[test]
        fn case_insensitive() {
            assert!(is_non_imperative("ADDED feature"));
            assert!(is_non_imperative("Fixed BUG"));
            assert!(is_non_imperative("UPDATING tests"));
        }

        #[test]
        fn does_not_match_mid_sentence() {
            // These should not trigger because the verb is not at the start
            assert!(!is_non_imperative("feat: Add updated timestamp #123"));
            assert!(!is_non_imperative("fix: Handle added complexity"));
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

        #[test]
        fn wip_commit_fails() {
            let commits = vec![create_valid_commit(
                "WIP: feat: add user authentication #123",
            )];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::WipCommit));
        }

        #[test]
        fn non_imperative_commit_fails() {
            let commits = vec![create_valid_commit("feat: Added user authentication #123")];
            let results = validate_commits(&commits, 10);
            assert_eq!(results.len(), 1);
            assert!(results[0].failures.contains(&Validation::NonImperative));
        }
    }
}
