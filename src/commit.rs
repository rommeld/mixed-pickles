//! Commit parsing and validation;
use crate::error::CLIError;
use pyo3::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// A git commit with its metadata.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Commit {
    hash: String,
    author_name: String,
    author_email: String,
    subject: String,
}

#[pymethods]
impl Commit {
    /// The commit hash.
    #[getter]
    fn hash(&self) -> &str {
        &self.hash
    }

    /// The commit author's name.
    #[getter]
    fn author_name(&self) -> &str {
        &self.author_name
    }

    /// The commit author's email address.
    #[getter]
    fn author_email(&self) -> &str {
        &self.author_email
    }

    /// The commit message subject line.
    #[getter]
    fn subject(&self) -> &str {
        &self.subject
    }

    /// Check if this commit's subject is shorter than the threshold.
    fn is_short(&self, threshold: usize) -> bool {
        self.subject.len() <= threshold
    }
}

#[allow(unused)]
enum Validation {
    ShortCommit,
    MissingReference,
    // VagueLanguage,
    // CommitFormat,
    // WipCommit,
    // NonImperative,
}

impl Commit {
    pub fn fetch_all(
        repo_path: Option<&PathBuf>,
        limit: Option<usize>,
    ) -> Result<Vec<Commit>, CLIError> {
        let mut args = vec![
            "log".to_string(),
            "--pretty=format:%H|%an|%ae|%s".to_string(),
        ];

        if let Some(n) = limit {
            args.push(format!("-n{}", n));
        }

        let mut command = Command::new("git");

        if let Some(path) = repo_path {
            command.current_dir(path);
        }

        let log_command = command.args(&args).output()?;

        if !log_command.status.success() {
            let stderr = String::from_utf8_lossy(&log_command.stderr);
            return Err(CLIError::GitCommandFailed(stderr.trim().to_string()));
        }

        let log_output = String::from_utf8_lossy(&log_command.stdout);
        let mut commits = Vec::new();

        for line in log_output.lines() {
            let parts: Vec<&str> = line.splitn(4, '|').collect();

            match parts.as_slice() {
                [hash, author_name, author_email, subject] => {
                    let commit = Commit {
                        hash: hash.to_string(),
                        author_name: author_name.to_string(),
                        author_email: author_email.to_string(),
                        subject: subject.to_string(),
                    };
                    commits.push(commit);
                }
                _ => {
                    eprintln!("Warning: Could not parse commit line");
                }
            }
        }

        Ok(commits)
    }

    pub fn find_short(commits: &[Commit], threshold: usize) -> Vec<(&str, &str)> {
        commits
            .iter()
            .filter(|c| c.is_short(threshold))
            .map(|c| (c.hash(), c.subject.as_str()))
            .collect()
    }
}

pub fn validate_repo_path(path: &Path) -> Result<(), CLIError> {
    if !path.exists() {
        return Err(CLIError::PathNotFound(path.to_path_buf()));
    }
    if !path.join(".git").exists() {
        return Err(CLIError::NotARepository(path.to_path_buf()));
    }
    Ok(())
}

enum CommitMessageStatus {
    NeedsWork,
    Acceptable,
    Empty,
}

impl CommitMessageStatus {
    fn from_short_commits(short_commits: &[(&str, &str)], total_commits: usize) -> Self {
        if total_commits == 0 {
            CommitMessageStatus::Empty
        } else if short_commits.is_empty() {
            CommitMessageStatus::Acceptable
        } else {
            CommitMessageStatus::NeedsWork
        }
    }
}

pub fn print_results(
    short_commits: &[(&str, &str)],
    total_commits: usize,
    analyzed_count: usize,
    threshold: usize,
    path: &Option<PathBuf>,
) {
    let status = CommitMessageStatus::from_short_commits(short_commits, total_commits);

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
            println!(
                "Found {} commits with short messages (< {} chars):",
                short_commits.len(),
                threshold
            );
            for (hash, subject) in short_commits {
                println!("  {}: \"{}\"", hash, subject);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod commit_struct {
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
            let commit =
                create_test_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "1234567890");
            assert!(commit.is_short(10));
        }
    }

    mod find_short {
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
        fn finds_short_commits() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, 10);
            assert_eq!(short.len(), 2);
            assert_eq!(short[0].1, "short");
            assert_eq!(short[1].1, "tiny");
        }

        #[test]
        fn returns_empty_when_no_short_commits() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, 3);
            assert!(short.is_empty());
        }

        #[test]
        fn returns_empty_for_empty_commits_list() {
            let commits: Vec<Commit> = vec![];
            let short = Commit::find_short(&commits, 10);
            assert!(short.is_empty());
        }
    }

    mod commit_message_status {
        use super::*;

        #[test]
        fn empty_when_no_commits() {
            let short_commits: Vec<(&str, &str)> = vec![];
            let status = CommitMessageStatus::from_short_commits(&short_commits, 0);
            assert!(matches!(status, CommitMessageStatus::Empty));
        }

        #[test]
        fn acceptable_when_no_short_commits() {
            let short_commits: Vec<(&str, &str)> = vec![];
            let status = CommitMessageStatus::from_short_commits(&short_commits, 10);
            assert!(matches!(status, CommitMessageStatus::Acceptable));
        }

        #[test]
        fn needs_work_when_short_commits_exist() {
            let short_commits: Vec<(&str, &str)> = vec![("abc1234", "fix")];
            let status = CommitMessageStatus::from_short_commits(&short_commits, 10);
            assert!(matches!(status, CommitMessageStatus::NeedsWork));
        }
    }

    mod validate_repo_path {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn returns_error_for_nonexistent_path() {
            let path = PathBuf::from("/this/path/does/not/exist");
            let result = validate_repo_path(&path);
            assert!(matches!(result, Err(CLIError::PathNotFound(_))));
        }

        #[test]
        fn returns_error_for_non_git_directory() {
            let path = PathBuf::from("/tmp");
            let result = validate_repo_path(&path);
            assert!(matches!(result, Err(CLIError::NotARepository(_))));
        }

        #[test]
        fn succeeds_for_current_git_repo() {
            let path = PathBuf::from(".");
            let result = validate_repo_path(&path);
            assert!(result.is_ok());
        }
    }
}
