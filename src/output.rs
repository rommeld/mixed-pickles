//! Output formatting for commit analysis results.

use std::path::PathBuf;

use crate::validation::{Severity, ValidationConfig, ValidationResult};

/// Status of commit message analysis.
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

/// Get the severity prefix for display.
fn severity_prefix(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "[error]",
        Severity::Warning => "[warn]",
        Severity::Info => "[info]",
        Severity::Ignore => "",
    }
}

/// Print validation results to stdout.
pub fn print_results(
    validation_results: &[ValidationResult],
    total_commits: usize,
    analyzed_count: usize,
    threshold: usize,
    path: &Option<PathBuf>,
    config: &ValidationConfig,
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
            println!(
                "Analyzed {} of {} total commits on path {:?}\n",
                analyzed_count, total_commits, path
            );

            // Count by severity
            let error_count = validation_results
                .iter()
                .filter(|r| r.failures.iter().any(|v| config.is_error(v)))
                .count();
            let warning_count = validation_results
                .iter()
                .filter(|r| {
                    r.failures
                        .iter()
                        .any(|v| config.get_severity(v) == Severity::Warning)
                })
                .count();

            println!(
                "Found {} commits with issues ({} errors, {} warnings) (threshold: {} chars):\n",
                validation_results.len(),
                error_count,
                warning_count,
                threshold
            );

            for result in validation_results {
                println!(
                    "  {}: \"{}\"",
                    result.commit.hash(),
                    result.commit.subject()
                );
                for failure in &result.failures {
                    let severity = config.get_severity(failure);
                    println!("    {} {}", severity_prefix(severity), failure);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::Commit;
    use crate::validation::Validation;

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
            failures: vec![Validation::ShortCommit],
        }];
        let status = CommitMessageStatus::from_validation_results(&results, 10);
        assert!(matches!(status, CommitMessageStatus::NeedsWork));
    }
}
