//! Output formatting.

use std::path::PathBuf;

use crate::validation::{Severity, ValidationResult};

enum CommitMessageStatus {
    NeedsWork,
    Acceptable,
    Empty,
}

impl CommitMessageStatus {
    fn from_validation_results(results: &[ValidationResult], total_commits: usize) -> Self {
        if total_commits == 0 {
            CommitMessageStatus::Empty
        } else if results.is_empty() {
            CommitMessageStatus::Acceptable
        } else {
            CommitMessageStatus::NeedsWork
        }
    }
}

fn severity_prefix(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "✗",
        Severity::Warning => "⚠",
        Severity::Info => "ℹ",
        Severity::Ignore => "",
    }
}

fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}

pub fn print_results(
    validation_results: &[ValidationResult],
    total_commits: usize,
    analyzed_count: usize,
    threshold: usize,
    path: &Option<PathBuf>,
) {
    let status = CommitMessageStatus::from_validation_results(validation_results, total_commits);

    match status {
        CommitMessageStatus::Empty => {
            println!("No commits found in repository.");
        }
        CommitMessageStatus::Acceptable => {
            println!("Commit messages are adequately executed.");
        }
        CommitMessageStatus::NeedsWork => {
            let path_display = path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            println!(
                "Analyzed {} of {} total in {} (threshold: {} chars)\n",
                pluralize(analyzed_count, "commit", "commits"),
                total_commits,
                path_display,
                threshold
            );

            let mut total_errors = 0;
            let mut total_warnings = 0;
            for result in validation_results {
                for finding in &result.findings {
                    match finding.severity {
                        Severity::Error => total_errors += 1,
                        Severity::Warning => total_warnings += 1,
                        _ => {}
                    }
                }
            }

            for result in validation_results {
                println!(
                    "Commit {} by {} <{}>",
                    result.commit.hash(),
                    result.commit.author_name(),
                    result.commit.author_email()
                );
                let subject = result.commit.subject();
                println!("  Subject: \"{}\"", subject);
                for finding in &result.findings {
                    println!(
                        "  {} {}",
                        severity_prefix(finding.severity),
                        finding.validation
                    );
                    let suggestion = finding.validation.suggest(subject);
                    println!("    → {}", suggestion);
                }
                println!();
            }

            println!(
                "Summary: {} with issues ({}, {})",
                pluralize(validation_results.len(), "commit", "commits"),
                pluralize(total_errors, "error", "errors"),
                pluralize(total_warnings, "warning", "warnings")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::Commit;
    use crate::validation::{Finding, Validation};

    fn create_test_commit(subject: &str) -> Commit {
        Commit {
            hash: "abc1234".to_string(),
            author_name: "Author".to_string(),
            author_email: "a@b.com".to_string(),
            subject: subject.to_string(),
        }
    }

    #[test]
    fn empty_when_no_commits() {
        let results: Vec<ValidationResult> = vec![];
        let status = CommitMessageStatus::from_validation_results(&results, 0);
        assert!(matches!(status, CommitMessageStatus::Empty));
    }

    #[test]
    fn acceptable_when_no_validation_failures() {
        let results: Vec<ValidationResult> = vec![];
        let status = CommitMessageStatus::from_validation_results(&results, 10);
        assert!(matches!(status, CommitMessageStatus::Acceptable));
    }

    #[test]
    fn needs_work_when_validation_failures_exist() {
        let commit = create_test_commit("fix");
        let results = vec![ValidationResult {
            commit: &commit,
            findings: vec![Finding::new(Validation::ShortCommit, Severity::Warning)],
        }];
        let status = CommitMessageStatus::from_validation_results(&results, 10);
        assert!(matches!(status, CommitMessageStatus::NeedsWork));
    }
}
