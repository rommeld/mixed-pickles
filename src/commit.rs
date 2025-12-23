//! Commit parsing and validationuse regex::Regex;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use crate::error::CLIError;

static GIT_HASH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{40}$").expect("Invalid regex pattern"));

fn is_valid_git_hash(hash: &str) -> bool {
    GIT_HASH_REGEX.is_match(hash)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Commit {
    pub hash: String,
    author_name: String,
    author_email: String,
    subject: String,
}

impl Commit {
    pub fn fetch_all(repo_path: Option<&PathBuf>) -> Result<Vec<Commit>, CLIError> {
        let args: Vec<String> = vec![
            "log".to_string(),
            "--pretty=format:%H|%an|%ae|%s".to_string(),
        ];

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
        let parsed_commit: Vec<&str> = log_output.lines().collect();

        let mut commits = Vec::new();

        for line in parsed_commit {
            let parts: Vec<&str> = line.splitn(4, '|').collect();

            match parts.as_slice() {
                [hash, author_name, author_email, subject] if is_valid_git_hash(hash) => {
                    let commit = Commit {
                        hash: hash.to_string(),
                        author_name: author_name.to_string(),
                        author_email: author_email.to_string(),
                        subject: subject.to_string(),
                    };
                    commits.push(commit);
                }
                [hash, _, _, _] => {
                    eprintln!("Warning: Invalid git hash format: {}", hash);
                }
                _ => {
                    eprintln!("Warning: Could not parse commit line");
                }
            }
        }

        Ok(commits)
    }

    fn is_short(&self, threshold: usize) -> bool {
        self.subject.len() <= threshold
    }

    fn short_hash(&self) -> &str {
        &self.hash[..7]
    }

    pub fn find_short(
        commits: &[Commit],
        limit: Option<usize>,
        threshold: usize,
    ) -> Vec<(&str, &str)> {
        let commits_to_check: Box<dyn Iterator<Item = &Commit>> = match limit {
            Some(n) => Box::new(commits.iter().take(n)),
            None => Box::new(commits.iter()),
        };

        commits_to_check
            .filter(|c| c.is_short(threshold))
            .map(|c| (c.short_hash(), c.subject.as_str()))
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

    mod git_hash_validation {
        use super::*;

        #[test]
        fn valid_40_char_lowercase_hex_hash() {
            let hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
            assert!(is_valid_git_hash(hash));
        }

        #[test]
        fn valid_hash_all_digits() {
            let hash = "1234567890123456789012345678901234567890";
            assert!(is_valid_git_hash(hash));
        }

        #[test]
        fn valid_hash_all_letters() {
            let hash = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
            assert!(is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_too_short() {
            let hash = "a1b2c3d4e5f6";
            assert!(!is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_too_long() {
            let hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3";
            assert!(!is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_uppercase_letters() {
            let hash = "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2";
            assert!(!is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_special_characters() {
            let hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b!";
            assert!(!is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_empty_string() {
            let hash = "";
            assert!(!is_valid_git_hash(hash));
        }

        #[test]
        fn invalid_hash_with_spaces() {
            let hash = "a1b2c3d4 5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
            assert!(!is_valid_git_hash(hash));
        }
    }

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

        #[test]
        fn short_hash_returns_first_seven_chars() {
            let commit = create_test_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "test");
            assert_eq!(commit.short_hash(), "a1b2c3d");
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
        fn finds_short_commits_without_limit() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, None, 10);
            assert_eq!(short.len(), 2);
            assert_eq!(short[0].1, "short");
            assert_eq!(short[1].1, "tiny");
        }

        #[test]
        fn respects_limit_parameter() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, Some(1), 10);
            assert_eq!(short.len(), 1);
            assert_eq!(short[0].1, "short");
        }

        #[test]
        fn returns_empty_when_no_short_commits() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, None, 3);
            assert!(short.is_empty());
        }

        #[test]
        fn returns_empty_for_empty_commits_list() {
            let commits: Vec<Commit> = vec![];
            let short = Commit::find_short(&commits, None, 10);
            assert!(short.is_empty());
        }

        #[test]
        fn limit_larger_than_commit_count() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, Some(100), 10);
            assert_eq!(short.len(), 2);
        }

        #[test]
        fn returns_short_hash_and_subject() {
            let commits = create_test_commits();
            let short = Commit::find_short(&commits, None, 10);
            assert_eq!(short[0].0, "1111111");
            assert_eq!(short[0].1, "short");
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
