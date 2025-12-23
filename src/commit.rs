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
