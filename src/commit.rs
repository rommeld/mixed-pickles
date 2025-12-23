use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::CLIError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Commit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub subject: String,
}

impl Commit {
    /// Fetches commits from the git repository at the given path
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

        for commit_message in parsed_commit.into_iter().map(|line| line.splitn(4, "|")) {
            let parts: Vec<&str> = commit_message.collect();

            if parts.len() == 4 {
                let commit = Commit {
                    hash: parts[0].to_string(),
                    author_name: parts[1].to_string(),
                    author_email: parts[2].to_string(),
                    subject: parts[3].to_string(),
                };
                commits.push(commit);
            } else {
                eprintln!("Warning: Could not parse commit line");
            }
        }

        Ok(commits)
    }

    /// Check if this commit has a short subject
    pub fn is_short(&self, threshold: usize) -> bool {
        self.subject.len() <= threshold
    }

    /// Get short hash (7 chars)
    pub fn short_hash(&self) -> &str {
        &self.hash[..7]
    }

    /// Finds commits with short subject lines
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

/// Validates that the given path exists and is a git repository
pub fn validate_repo_path(path: &Path) -> Result<(), CLIError> {
    if !path.exists() {
        return Err(CLIError::PathNotFound(path.to_path_buf()));
    }
    if !path.join(".git").exists() {
        return Err(CLIError::NotARepository(path.to_path_buf()));
    }
    Ok(())
}

/// Prints the analysis results
pub fn print_results(
    short_commits: &[(&str, &str)],
    total_commits: usize,
    analyzed_count: usize,
    threshold: usize,
    path: &Option<PathBuf>,
) {
    if short_commits.is_empty() {
        println!("Commit messages are adequately executed.");
    } else {
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
