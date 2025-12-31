//! Validation types and logic.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use pyo3::prelude::*;
use regex::Regex;

use crate::config::{ConfigError, SeverityConfig, ToolConfig};

static REFERENCE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(#\d+|gh-\d+|[A-Z]{2,}-\d+)").expect("Invalid reference regex")
});

static CONVENTIONAL_COMMIT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?!?:\s.+")
        .expect("Invalid conventional commit regex")
});

static VAGUE_LANGUAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(fix(ed|es|ing)?|update[ds]?|change[ds]?|modify|modified|modifies|tweak(ed|s)?|adjust(ed|s)?)\s+(it|this|that|things?|stuff|code|bug|issue|error|problem)s?\b")
        .expect("Invalid vague language regex")
});

static WIP_COMMIT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(^wip\b|^wip:|^\[wip\]|\bwork.?in.?progress\b|^fixup!|^squash!|^amend!|\bdo\s*not\s*merge\b|\bdon'?t\s*merge\b|\bwip\s*$)")
        .expect("Invalid WIP commit regex")
});

static NON_IMPERATIVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
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

impl Validation {
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

    pub fn suggest(&self, subject: &str) -> String {
        match self {
            Validation::ShortCommit => "Add context: what changed and why".to_string(),
            Validation::MissingReference => {
                "Add a ticket reference (e.g., #123 or PROJ-456) if applicable".to_string()
            }
            Validation::InvalidFormat => {
                let suggested_type = suggest_commit_type(subject);
                format!(
                    "Use conventional format: '{}: <description>'",
                    suggested_type
                )
            }
            Validation::VagueLanguage => match find_vague_language(subject) {
                Some(vague_phrase) => format!(
                    "'{}' lacks specifics - mention what and where",
                    vague_phrase
                ),
                None => "Be specific about what changed and where".to_string(),
            },
            Validation::WipCommit => get_wip_suggestion(subject),
            Validation::NonImperative => match find_non_imperative(subject) {
                Some(before) => {
                    let after = to_imperative(before);
                    let after = if before.chars().next().is_some_and(|c| c.is_uppercase()) {
                        let mut chars = after.chars();
                        match chars.next() {
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                            None => after,
                        }
                    } else {
                        after
                    };
                    format!("Use imperative: '{}' → '{}'", before, after)
                }
                None => "Use imperative mood (e.g., 'Add' not 'Added')".to_string(),
            },
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
    Error,
    Warning,
    Info,
    Ignore,
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" | "warn" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            "ignore" | "off" => Ok(Severity::Ignore),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub validation: Validation,
    pub severity: Severity,
}

impl Finding {
    pub fn new(validation: Validation, severity: Severity) -> Self {
        Self {
            validation,
            severity,
        }
    }
}

/// Configure which validations run and their severity levels.
#[pyclass]
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    severities: HashMap<Validation, Severity>,
    #[pyo3(get, set)]
    pub threshold: usize,
    #[pyo3(get, set)]
    pub check_short: bool,
    #[pyo3(get, set)]
    pub require_issue_ref: bool,
    #[pyo3(get, set)]
    pub require_conventional_format: bool,
    #[pyo3(get, set)]
    pub check_vague_language: bool,
    #[pyo3(get, set)]
    pub check_wip: bool,
    #[pyo3(get, set)]
    pub check_imperative: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut severities = HashMap::new();
        severities.insert(Validation::WipCommit, Severity::Error);
        severities.insert(Validation::ShortCommit, Severity::Warning);
        severities.insert(Validation::VagueLanguage, Severity::Warning);
        severities.insert(Validation::NonImperative, Severity::Warning);
        severities.insert(Validation::MissingReference, Severity::Info);
        severities.insert(Validation::InvalidFormat, Severity::Info);
        Self {
            severities,
            threshold: 30,
            check_short: true,
            require_issue_ref: true,
            require_conventional_format: true,
            check_vague_language: true,
            check_wip: true,
            check_imperative: true,
        }
    }
}

impl ValidationConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    #[must_use]
    pub fn check_short(mut self, check: bool) -> Self {
        self.check_short = check;
        self
    }

    #[must_use]
    pub fn require_issue_ref(mut self, require: bool) -> Self {
        self.require_issue_ref = require;
        self
    }

    #[must_use]
    pub fn require_conventional_format(mut self, require: bool) -> Self {
        self.require_conventional_format = require;
        self
    }

    #[must_use]
    pub fn check_vague_language(mut self, check: bool) -> Self {
        self.check_vague_language = check;
        self
    }

    #[must_use]
    pub fn check_wip(mut self, check: bool) -> Self {
        self.check_wip = check;
        self
    }

    #[must_use]
    pub fn check_imperative(mut self, check: bool) -> Self {
        self.check_imperative = check;
        self
    }

    #[must_use]
    pub fn severity(mut self, validation: Validation, severity: Severity) -> Self {
        self.severities.insert(validation, severity);
        self
    }

    pub fn get_severity(&self, validation: &Validation) -> Severity {
        self.severities
            .get(validation)
            .copied()
            .unwrap_or(Severity::Warning)
    }

    pub fn should_report(&self, validation: &Validation) -> bool {
        self.get_severity(validation) != Severity::Ignore
    }

    pub fn is_error(&self, validation: &Validation) -> bool {
        self.get_severity(validation) == Severity::Error
    }

    pub fn parse_and_set(&mut self, validations: &str, severity: Severity) -> Result<(), String> {
        for name in validations.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let validation = Validation::from_str(name)?;
            self.severities.insert(validation, severity);
        }
        Ok(())
    }

    pub fn parse_and_disable(&mut self, validations: &str) -> Result<(), String> {
        for name in validations.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            match name.to_lowercase().as_str() {
                "shortcommit" | "short" => self.check_short = false,
                "missingreference" | "reference" | "ref" => self.require_issue_ref = false,
                "invalidformat" | "format" => self.require_conventional_format = false,
                "vaguelanguage" | "vague" => self.check_vague_language = false,
                "wipcommit" | "wip" => self.check_wip = false,
                "nonimperative" | "imperative" => self.check_imperative = false,
                _ => return Err(format!("Unknown validation type: {}", name)),
            }
        }
        Ok(())
    }

    /// Apply configuration from a file (pyproject.toml or .mixed-pickles.toml).
    pub fn apply_file_config(&mut self, config: &ToolConfig) -> Result<(), ConfigError> {
        if let Some(threshold) = config.threshold {
            self.threshold = threshold;
        }

        for name in &config.disable {
            self.disable_validation(name)?;
        }

        if let Some(ref severity_config) = config.severity {
            self.apply_severity_config(severity_config)?;
        }

        Ok(())
    }

    fn disable_validation(&mut self, name: &str) -> Result<(), ConfigError> {
        match name.to_lowercase().replace('-', "").as_str() {
            "shortcommit" | "short" => self.check_short = false,
            "missingreference" | "reference" | "ref" => self.require_issue_ref = false,
            "invalidformat" | "format" => self.require_conventional_format = false,
            "vaguelanguage" | "vague" => self.check_vague_language = false,
            "wipcommit" | "wip" => self.check_wip = false,
            "nonimperative" | "imperative" => self.check_imperative = false,
            _ => return Err(ConfigError::InvalidValidation(name.to_string())),
        }
        Ok(())
    }

    fn apply_severity_config(&mut self, config: &SeverityConfig) -> Result<(), ConfigError> {
        let mappings: [(&Option<String>, Validation); 6] = [
            (&config.short, Validation::ShortCommit),
            (&config.wip, Validation::WipCommit),
            (&config.reference, Validation::MissingReference),
            (&config.format, Validation::InvalidFormat),
            (&config.vague, Validation::VagueLanguage),
            (&config.imperative, Validation::NonImperative),
        ];

        for (severity_opt, validation) in mappings {
            if let Some(s) = severity_opt {
                self.set_severity_from_str(validation, s)?;
            }
        }
        Ok(())
    }

    fn set_severity_from_str(
        &mut self,
        validation: Validation,
        severity_str: &str,
    ) -> Result<(), ConfigError> {
        let severity = Severity::from_str(severity_str)
            .map_err(|_| ConfigError::InvalidSeverity(severity_str.to_string()))?;
        self.severities.insert(validation, severity);
        Ok(())
    }
}

#[pymethods]
impl ValidationConfig {
    /// Create a new ValidationConfig.
    ///
    /// Args:
    ///     threshold: Minimum message length in characters (default: 30)
    ///     check_short: Check for short commit messages (default: True)
    ///     require_issue_ref: Check for issue references like #123 (default: True)
    ///     require_conventional_format: Check for conventional commit format (default: True)
    ///     check_vague_language: Check for vague descriptions (default: True)
    ///     check_wip: Check for WIP/fixup commits (default: True)
    ///     check_imperative: Check for imperative mood (default: True)
    #[new]
    #[pyo3(signature = (
        threshold=30,
        check_short=true,
        require_issue_ref=true,
        require_conventional_format=true,
        check_vague_language=true,
        check_wip=true,
        check_imperative=true
    ))]
    fn py_new(
        threshold: usize,
        check_short: bool,
        require_issue_ref: bool,
        require_conventional_format: bool,
        check_vague_language: bool,
        check_wip: bool,
        check_imperative: bool,
    ) -> Self {
        Self {
            threshold,
            check_short,
            require_issue_ref,
            require_conventional_format,
            check_vague_language,
            check_wip,
            check_imperative,
            ..Default::default()
        }
    }

    /// Set the severity level for a validation type.
    #[pyo3(name = "set_severity")]
    fn py_set_severity(&mut self, validation: Validation, severity: Severity) {
        self.severities.insert(validation, severity);
    }

    /// Get the severity level for a validation type.
    #[pyo3(name = "get_severity")]
    fn py_get_severity(&self, validation: Validation) -> Severity {
        self.get_severity(&validation)
    }

    /// Load configuration from a file.
    ///
    /// Args:
    ///     path: Path to pyproject.toml or .mixed-pickles.toml
    ///
    /// Returns:
    ///     ValidationConfig with settings from the file
    ///
    /// Raises:
    ///     RuntimeError: If the file cannot be read or parsed
    #[staticmethod]
    fn from_file(path: String) -> PyResult<Self> {
        use crate::config::{ConfigFile, load_config};
        use std::path::PathBuf;

        let path_buf = PathBuf::from(&path);
        let config_file = if path.ends_with("pyproject.toml") {
            ConfigFile::PyProjectToml(path_buf)
        } else {
            ConfigFile::Dedicated(path_buf)
        };

        let file_config = load_config(&config_file)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let mut config = Self::default();
        config
            .apply_file_config(&file_config)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(config)
    }

    /// Discover and load configuration from a directory.
    ///
    /// Searches for pyproject.toml or .mixed-pickles.toml starting from
    /// the given directory and walking up to parent directories.
    ///
    /// Args:
    ///     path: Directory to start searching from (default: current directory)
    ///
    /// Returns:
    ///     ValidationConfig with discovered settings, or defaults if no config found
    ///
    /// Raises:
    ///     RuntimeError: If the config file exists but cannot be parsed
    #[staticmethod]
    #[pyo3(signature = (path=None))]
    fn discover(path: Option<String>) -> PyResult<Self> {
        use crate::config::{find_config_file, load_config};
        use std::path::Path;

        let start_dir = path
            .as_ref()
            .map(|p| Path::new(p.as_str()))
            .unwrap_or(Path::new("."));

        let mut config = Self::default();

        if let Some(config_file) = find_config_file(start_dir) {
            let file_config = load_config(&config_file)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            config
                .apply_file_config(&file_config)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }

        Ok(config)
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationConfig(threshold={}, require_issue_ref={}, require_conventional_format={}, check_vague_language={}, check_wip={}, check_imperative={})",
            self.threshold,
            self.require_issue_ref,
            self.require_conventional_format,
            self.check_vague_language,
            self.check_wip,
            self.check_imperative
        )
    }
}

#[pymethods]
impl Validation {
    /// Human-readable description of this validation type.
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

pub fn has_reference(subject: &str) -> bool {
    REFERENCE_REGEX.is_match(subject)
}

pub fn has_conventional_format(subject: &str) -> bool {
    CONVENTIONAL_COMMIT_REGEX.is_match(subject)
}

pub fn find_vague_language(subject: &str) -> Option<&str> {
    VAGUE_LANGUAGE_REGEX.find(subject).map(|m| m.as_str())
}

pub fn is_wip_commit(subject: &str) -> bool {
    WIP_COMMIT_REGEX.is_match(subject)
}

pub fn find_non_imperative(subject: &str) -> Option<&str> {
    NON_IMPERATIVE_REGEX
        .captures(subject)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
}

fn check_short(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if !config.check_short {
        return None;
    }
    if subject.len() <= config.threshold {
        let severity = config.get_severity(&Validation::ShortCommit);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::ShortCommit, severity));
        }
    }
    None
}

fn check_issue_reference(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if !has_reference(subject) {
        let severity = config.get_severity(&Validation::MissingReference);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::MissingReference, severity));
        }
    }
    None
}

fn suggest_commit_type(subject: &str) -> &'static str {
    let lower = subject.to_lowercase();

    if lower.starts_with("merge") {
        return "chore";
    }
    if lower.starts_with("fix")
        || lower.contains("resolve")
        || lower.contains("repair")
        || lower.contains("patch")
        || lower.contains("correct")
    {
        return "fix";
    }
    if lower.starts_with("add")
        || lower.starts_with("implement")
        || lower.starts_with("create")
        || lower.starts_with("introduce")
        || lower.contains("new feature")
    {
        return "feat";
    }
    if lower.contains("refactor")
        || lower.contains("restructure")
        || lower.contains("reorganize")
        || lower.contains("simplify")
        || lower.contains("clean up")
    {
        return "refactor";
    }
    if lower.starts_with("delete")
        || lower.starts_with("remove")
        || lower.starts_with("init")
        || lower.contains("bump")
        || lower.contains("upgrade")
        || lower.contains("update dep")
        || lower.contains("dependency")
        || lower.contains("version")
        || lower.contains("config")
    {
        return "chore";
    }
    if lower.contains("readme")
        || lower.contains("doc")
        || lower.contains("comment")
        || lower.contains("typo")
    {
        return "docs";
    }
    if lower.contains("test") || lower.contains("spec") || lower.contains("coverage") {
        return "test";
    }
    "feat"
}

fn check_conventional_format(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if !has_conventional_format(subject) {
        let severity = config.get_severity(&Validation::InvalidFormat);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::InvalidFormat, severity));
        }
    }
    None
}

fn check_vague(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if find_vague_language(subject).is_some() {
        let severity = config.get_severity(&Validation::VagueLanguage);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::VagueLanguage, severity));
        }
    }
    None
}

fn get_wip_suggestion(subject: &str) -> String {
    let lower = subject.to_lowercase();

    if lower.starts_with("fixup!") {
        return "Squash before merging: git rebase -i --autosquash".to_string();
    }
    if lower.starts_with("squash!") {
        return "Squash before merging: git rebase -i --autosquash".to_string();
    }
    if lower.starts_with("amend!") {
        return "Rebase before merging: git rebase -i --autosquash".to_string();
    }
    if lower.contains("do not merge")
        || lower.contains("don't merge")
        || lower.contains("dont merge")
    {
        return "Remove marker when ready: git commit --amend".to_string();
    }
    "Finalize before merging: git commit --amend".to_string()
}

fn check_wip(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if is_wip_commit(subject) {
        let severity = config.get_severity(&Validation::WipCommit);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::WipCommit, severity));
        }
    }
    None
}

static IMPERATIVE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("added", "add"),
        ("removed", "remove"),
        ("fixed", "fix"),
        ("updated", "update"),
        ("changed", "change"),
        ("implemented", "implement"),
        ("created", "create"),
        ("deleted", "delete"),
        ("modified", "modify"),
        ("refactored", "refactor"),
        ("improved", "improve"),
        ("resolved", "resolve"),
        ("merged", "merge"),
        ("moved", "move"),
        ("renamed", "rename"),
        ("replaced", "replace"),
        ("cleaned", "clean"),
        ("enabled", "enable"),
        ("disabled", "disable"),
        ("converted", "convert"),
        ("introduced", "introduce"),
        ("integrated", "integrate"),
        ("adjusted", "adjust"),
        ("corrected", "correct"),
        ("enhanced", "enhance"),
        ("extended", "extend"),
        ("optimized", "optimize"),
        ("simplified", "simplify"),
        ("upgraded", "upgrade"),
        ("migrated", "migrate"),
        ("adding", "add"),
        ("removing", "remove"),
        ("fixing", "fix"),
        ("updating", "update"),
        ("changing", "change"),
        ("implementing", "implement"),
        ("creating", "create"),
        ("deleting", "delete"),
        ("modifying", "modify"),
        ("refactoring", "refactor"),
        ("improving", "improve"),
        ("resolving", "resolve"),
        ("merging", "merge"),
        ("moving", "move"),
        ("renaming", "rename"),
        ("replacing", "replace"),
        ("cleaning", "clean"),
        ("enabling", "enable"),
        ("disabling", "disable"),
        ("converting", "convert"),
        ("introducing", "introduce"),
        ("integrating", "integrate"),
        ("adjusting", "adjust"),
        ("correcting", "correct"),
        ("enhancing", "enhance"),
        ("extending", "extend"),
        ("optimizing", "optimize"),
        ("simplifying", "simplify"),
        ("upgrading", "upgrade"),
        ("migrating", "migrate"),
    ])
});

fn to_imperative(word: &str) -> String {
    let lower = word.to_lowercase();
    IMPERATIVE_MAP
        .get(lower.as_str())
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| word.to_string())
}

fn check_imperative(subject: &str, config: &ValidationConfig) -> Option<Finding> {
    if find_non_imperative(subject).is_some() {
        let severity = config.get_severity(&Validation::NonImperative);
        if severity != Severity::Ignore {
            return Some(Finding::new(Validation::NonImperative, severity));
        }
    }
    None
}

use crate::commit::Commit;

/// Validate a commit against enabled checks.
///
/// Priority rules:
/// - WIP suppresses ShortCommit and VagueLanguage
/// - ShortCommit suppresses NonImperative
pub fn validate_commit(commit: &Commit, config: &ValidationConfig) -> Vec<Finding> {
    let mut findings = Vec::new();
    let subject = &commit.subject;

    let is_wip = config.check_wip && is_wip_commit(subject);
    if is_wip && let Some(f) = check_wip(subject, config) {
        findings.push(f);
    }

    let is_short = !is_wip && subject.len() <= config.threshold;
    if !is_wip {
        if let Some(f) = check_short(subject, config) {
            findings.push(f);
        }
        if config.check_vague_language
            && let Some(f) = check_vague(subject, config)
        {
            findings.push(f);
        }
    }

    if config.require_issue_ref
        && let Some(f) = check_issue_reference(subject, config)
    {
        findings.push(f);
    }

    if config.require_conventional_format
        && let Some(f) = check_conventional_format(subject, config)
    {
        findings.push(f);
    }

    if config.check_imperative
        && !is_short
        && let Some(f) = check_imperative(subject, config)
    {
        findings.push(f);
    }

    findings
}

/// A commit paired with its validation findings.
#[derive(Debug)]
pub struct ValidationResult<'a> {
    pub commit: &'a Commit,
    pub findings: Vec<Finding>,
}

impl<'a> ValidationResult<'a> {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == Severity::Warning)
    }
}

pub fn validate_commits<'a>(
    commits: &'a [Commit],
    config: &ValidationConfig,
) -> Vec<ValidationResult<'a>> {
    commits
        .iter()
        .map(|commit| ValidationResult {
            commit,
            findings: validate_commit(commit, config),
        })
        .filter(|result| !result.findings.is_empty())
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
            assert!(find_vague_language("fix bug").is_some());
            assert!(find_vague_language("fixed bug").is_some());
            assert!(find_vague_language("fixes bug").is_some());
            assert!(find_vague_language("fixing bug").is_some());
        }

        #[test]
        fn detects_update_code() {
            assert!(find_vague_language("update code").is_some());
            assert!(find_vague_language("updated code").is_some());
            assert!(find_vague_language("updates code").is_some());
        }

        #[test]
        fn detects_change_stuff() {
            assert!(find_vague_language("change stuff").is_some());
            assert!(find_vague_language("changed things").is_some());
            assert!(find_vague_language("changes it").is_some());
        }

        #[test]
        fn detects_modify_patterns() {
            assert!(find_vague_language("modify code").is_some());
            assert!(find_vague_language("modified this").is_some());
            assert!(find_vague_language("modifies that").is_some());
        }

        #[test]
        fn detects_tweak_adjust_patterns() {
            assert!(find_vague_language("tweak code").is_some());
            assert!(find_vague_language("tweaked stuff").is_some());
            assert!(find_vague_language("adjust things").is_some());
            assert!(find_vague_language("adjusted it").is_some());
        }

        #[test]
        fn detects_issue_error_problem() {
            assert!(find_vague_language("fix issue").is_some());
            assert!(find_vague_language("fix error").is_some());
            assert!(find_vague_language("fix problem").is_some());
            assert!(find_vague_language("fixed issues").is_some());
        }

        #[test]
        fn allows_specific_descriptions() {
            assert!(find_vague_language("fix: resolve null pointer in user login").is_none());
            assert!(find_vague_language("fix: handle edge case in parser").is_none());
            assert!(find_vague_language("update README with installation steps").is_none());
            assert!(find_vague_language("change default timeout to 30 seconds").is_none());
        }

        #[test]
        fn allows_conventional_commits_with_context() {
            assert!(find_vague_language("feat: add user authentication with OAuth2").is_none());
            assert!(find_vague_language("fix: resolve memory leak in connection pool").is_none());
            assert!(find_vague_language("docs: update API documentation").is_none());
        }

        #[test]
        fn case_insensitive() {
            assert!(find_vague_language("FIX BUG").is_some());
            assert!(find_vague_language("Fix Bug").is_some());
            assert!(find_vague_language("UPDATE CODE").is_some());
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
            assert!(find_non_imperative("Added new feature").is_some());
            assert!(find_non_imperative("added user authentication").is_some());
            assert!(find_non_imperative("feat: Added new endpoint").is_some());
        }

        #[test]
        fn detects_past_tense_fixed() {
            assert!(find_non_imperative("Fixed bug in parser").is_some());
            assert!(find_non_imperative("fix: Fixed memory leak").is_some());
        }

        #[test]
        fn detects_past_tense_updated() {
            assert!(find_non_imperative("Updated dependencies").is_some());
            assert!(find_non_imperative("docs: Updated README").is_some());
        }

        #[test]
        fn detects_past_tense_removed() {
            assert!(find_non_imperative("Removed unused code").is_some());
            assert!(find_non_imperative("refactor: Removed dead code").is_some());
        }

        #[test]
        fn detects_past_tense_other_verbs() {
            assert!(find_non_imperative("Changed configuration").is_some());
            assert!(find_non_imperative("Implemented feature").is_some());
            assert!(find_non_imperative("Created new module").is_some());
            assert!(find_non_imperative("Deleted old files").is_some());
            assert!(find_non_imperative("Modified settings").is_some());
            assert!(find_non_imperative("Refactored code").is_some());
            assert!(find_non_imperative("Improved performance").is_some());
            assert!(find_non_imperative("Resolved conflict").is_some());
            assert!(find_non_imperative("Merged branch").is_some());
            assert!(find_non_imperative("Moved files").is_some());
            assert!(find_non_imperative("Renamed variable").is_some());
        }

        #[test]
        fn detects_present_continuous() {
            assert!(find_non_imperative("Adding new feature").is_some());
            assert!(find_non_imperative("Fixing bug").is_some());
            assert!(find_non_imperative("Updating tests").is_some());
            assert!(find_non_imperative("Removing unused imports").is_some());
            assert!(find_non_imperative("feat: Implementing auth").is_some());
        }

        #[test]
        fn allows_imperative_mood() {
            assert!(find_non_imperative("Add new feature").is_none());
            assert!(find_non_imperative("Fix bug in parser").is_none());
            assert!(find_non_imperative("Update dependencies").is_none());
            assert!(find_non_imperative("Remove unused code").is_none());
            assert!(find_non_imperative("feat: Add user authentication").is_none());
            assert!(find_non_imperative("fix: Resolve memory leak").is_none());
        }

        #[test]
        fn allows_imperative_with_conventional_prefix() {
            assert!(find_non_imperative("feat: Add new endpoint").is_none());
            assert!(find_non_imperative("fix(api): Handle edge case").is_none());
            assert!(find_non_imperative("docs: Update README").is_none());
            assert!(find_non_imperative("refactor!: Simplify logic").is_none());
        }

        #[test]
        fn case_insensitive() {
            assert!(find_non_imperative("ADDED feature").is_some());
            assert!(find_non_imperative("Fixed BUG").is_some());
            assert!(find_non_imperative("UPDATING tests").is_some());
        }

        #[test]
        fn does_not_match_mid_sentence() {
            // These should not trigger because the verb is not at the start
            assert!(find_non_imperative("feat: Add updated timestamp #123").is_none());
            assert!(find_non_imperative("fix: Handle added complexity").is_none());
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

        fn config_with_threshold(threshold: usize) -> ValidationConfig {
            ValidationConfig::with_threshold(threshold)
        }

        fn has_validation(result: &ValidationResult, validation: Validation) -> bool {
            result.findings.iter().any(|f| f.validation == validation)
        }

        #[test]
        fn valid_commit_passes_all_validations() {
            let commits = vec![create_valid_commit(
                "feat: add new feature for user authentication #123",
            )];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert!(results.is_empty(), "Expected no failures for valid commit");
        }

        #[test]
        fn commit_missing_reference_fails() {
            let commits = vec![create_valid_commit("feat: add new feature")];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::MissingReference));
        }

        #[test]
        fn commit_with_invalid_format_fails() {
            let commits = vec![create_valid_commit("add new feature #123")];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::InvalidFormat));
        }

        #[test]
        fn short_commit_fails() {
            let commits = vec![create_valid_commit("feat: x #1")];
            let config = config_with_threshold(20);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::ShortCommit));
        }

        #[test]
        fn commit_can_have_multiple_failures() {
            let commits = vec![create_valid_commit("bad")];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::ShortCommit));
            assert!(has_validation(&results[0], Validation::MissingReference));
            assert!(has_validation(&results[0], Validation::InvalidFormat));
        }

        #[test]
        fn commit_with_vague_language_fails() {
            let commits = vec![create_valid_commit("feat: fix bug #123")];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::VagueLanguage));
        }

        #[test]
        fn wip_commit_fails() {
            let commits = vec![create_valid_commit(
                "WIP: feat: add user authentication #123",
            )];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::WipCommit));
        }

        #[test]
        fn non_imperative_commit_fails() {
            let commits = vec![create_valid_commit("feat: Added user authentication #123")];
            let config = config_with_threshold(10);
            let results = validate_commits(&commits, &config);
            assert_eq!(results.len(), 1);
            assert!(has_validation(&results[0], Validation::NonImperative));
        }
    }

    mod validate_commit_tests {
        use super::*;

        fn create_commit(subject: &str) -> Commit {
            Commit {
                hash: "abc123".to_string(),
                author_name: "Test".to_string(),
                author_email: "test@example.com".to_string(),
                subject: subject.to_string(),
            }
        }

        #[test]
        fn valid_commit_returns_no_findings() {
            let commit = create_commit("feat: add user authentication #123");
            let config = ValidationConfig::default();
            let findings = validate_commit(&commit, &config);
            assert!(findings.is_empty());
        }

        #[test]
        fn respects_threshold_config() {
            let commit = create_commit("feat: add user authentication #123");
            let config = ValidationConfig::default().threshold(100); // Very high threshold
            let findings = validate_commit(&commit, &config);
            assert!(
                findings
                    .iter()
                    .any(|f| f.validation == Validation::ShortCommit)
            );
        }

        #[test]
        fn disabled_issue_ref_check_skips_validation() {
            let commit = create_commit("feat: add feature without reference");
            let config = ValidationConfig::default().require_issue_ref(false);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::MissingReference)
            );
        }

        #[test]
        fn disabled_conventional_format_skips_validation() {
            let commit = create_commit("add feature without conventional format #123");
            let config = ValidationConfig::default().require_conventional_format(false);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::InvalidFormat)
            );
        }

        #[test]
        fn disabled_vague_language_skips_validation() {
            let commit = create_commit("feat: fix bug #123");
            let config = ValidationConfig::default().check_vague_language(false);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::VagueLanguage)
            );
        }

        #[test]
        fn disabled_wip_check_skips_validation() {
            let commit = create_commit("WIP: feat: add feature #123");
            let config = ValidationConfig::default().check_wip(false);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::WipCommit)
            );
        }

        #[test]
        fn disabled_imperative_check_skips_validation() {
            let commit = create_commit("feat: Added new feature #123");
            let config = ValidationConfig::default().check_imperative(false);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::NonImperative)
            );
        }

        #[test]
        fn findings_include_correct_severity() {
            // Test WIP severity
            let wip_commit = create_commit("WIP");
            let config = ValidationConfig::default();
            let wip_findings = validate_commit(&wip_commit, &config);

            let wip_finding = wip_findings
                .iter()
                .find(|f| f.validation == Validation::WipCommit);
            assert!(wip_finding.is_some());
            assert_eq!(wip_finding.unwrap().severity, Severity::Error);

            // WIP commits suppress ShortCommit check (priority rule)
            assert!(
                !wip_findings
                    .iter()
                    .any(|f| f.validation == Validation::ShortCommit)
            );

            // Test ShortCommit severity with a non-WIP commit
            let short_commit = create_commit("fix");
            let short_findings = validate_commit(&short_commit, &config);

            let short_finding = short_findings
                .iter()
                .find(|f| f.validation == Validation::ShortCommit);
            assert!(short_finding.is_some());
            assert_eq!(short_finding.unwrap().severity, Severity::Warning);
        }

        #[test]
        fn ignored_severity_excludes_finding() {
            let commit = create_commit("feat: add feature"); // Missing reference
            let config = ValidationConfig::default()
                .severity(Validation::MissingReference, Severity::Ignore);
            let findings = validate_commit(&commit, &config);
            assert!(
                !findings
                    .iter()
                    .any(|f| f.validation == Validation::MissingReference)
            );
        }

        #[test]
        fn custom_severity_is_reflected_in_finding() {
            let commit = create_commit("feat: add feature"); // Missing reference
            let config =
                ValidationConfig::default().severity(Validation::MissingReference, Severity::Error);
            let findings = validate_commit(&commit, &config);
            let finding = findings
                .iter()
                .find(|f| f.validation == Validation::MissingReference);
            assert!(finding.is_some());
            assert_eq!(finding.unwrap().severity, Severity::Error);
        }
    }

    mod apply_file_config_tests {
        use super::*;
        use crate::config::{SeverityConfig, ToolConfig};

        #[test]
        fn applies_threshold() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                threshold: Some(50),
                ..Default::default()
            };

            config.apply_file_config(&file_config).unwrap();
            assert_eq!(config.threshold, 50);
        }

        #[test]
        fn applies_disable_list() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                disable: vec!["short".to_string(), "wip".to_string()],
                ..Default::default()
            };

            config.apply_file_config(&file_config).unwrap();
            assert!(!config.check_short);
            assert!(!config.check_wip);
            assert!(config.check_vague_language); // unchanged
        }

        #[test]
        fn applies_severity_overrides() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                severity: Some(SeverityConfig {
                    short: Some("error".to_string()),
                    wip: Some("warning".to_string()),
                    reference: Some("ignore".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            config.apply_file_config(&file_config).unwrap();
            assert_eq!(
                config.get_severity(&Validation::ShortCommit),
                Severity::Error
            );
            assert_eq!(
                config.get_severity(&Validation::WipCommit),
                Severity::Warning
            );
            assert_eq!(
                config.get_severity(&Validation::MissingReference),
                Severity::Ignore
            );
        }

        #[test]
        fn handles_hyphenated_validation_names() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                disable: vec!["short-commit".to_string(), "missing-reference".to_string()],
                ..Default::default()
            };

            config.apply_file_config(&file_config).unwrap();
            assert!(!config.check_short);
            assert!(!config.require_issue_ref);
        }

        #[test]
        fn rejects_invalid_validation_name() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                disable: vec!["nonexistent".to_string()],
                ..Default::default()
            };

            let result = config.apply_file_config(&file_config);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                crate::config::ConfigError::InvalidValidation(_)
            ));
        }

        #[test]
        fn rejects_invalid_severity() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                severity: Some(SeverityConfig {
                    short: Some("critical".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            let result = config.apply_file_config(&file_config);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                crate::config::ConfigError::InvalidSeverity(_)
            ));
        }

        #[test]
        fn applies_all_config_options_together() {
            let mut config = ValidationConfig::default();
            let file_config = ToolConfig {
                threshold: Some(75),
                strict: Some(true),
                disable: vec!["format".to_string()],
                severity: Some(SeverityConfig {
                    short: Some("error".to_string()),
                    ..Default::default()
                }),
            };

            config.apply_file_config(&file_config).unwrap();
            assert_eq!(config.threshold, 75);
            assert!(!config.require_conventional_format);
            assert_eq!(
                config.get_severity(&Validation::ShortCommit),
                Severity::Error
            );
        }
    }

    mod severity_from_str_tests {
        use super::*;

        #[test]
        fn parses_error() {
            assert_eq!(Severity::from_str("error").unwrap(), Severity::Error);
            assert_eq!(Severity::from_str("ERROR").unwrap(), Severity::Error);
        }

        #[test]
        fn parses_warning() {
            assert_eq!(Severity::from_str("warning").unwrap(), Severity::Warning);
            assert_eq!(Severity::from_str("warn").unwrap(), Severity::Warning);
        }

        #[test]
        fn parses_info() {
            assert_eq!(Severity::from_str("info").unwrap(), Severity::Info);
            assert_eq!(Severity::from_str("INFO").unwrap(), Severity::Info);
        }

        #[test]
        fn parses_ignore() {
            assert_eq!(Severity::from_str("ignore").unwrap(), Severity::Ignore);
            assert_eq!(Severity::from_str("off").unwrap(), Severity::Ignore);
        }

        #[test]
        fn rejects_invalid() {
            assert!(Severity::from_str("critical").is_err());
            assert!(Severity::from_str("").is_err());
        }
    }
}
