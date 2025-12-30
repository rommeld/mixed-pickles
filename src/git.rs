//! Git operations.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::commit::Commit;
use crate::error::CLIError;

pub fn fetch_commits(
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

pub fn count_commits(repo_path: Option<&PathBuf>) -> Result<usize, CLIError> {
    let mut command = Command::new("git");

    if let Some(path) = repo_path {
        command.current_dir(path);
    }

    let output = command.args(["rev-list", "--count", "HEAD"]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CLIError::GitCommandFailed(stderr.trim().to_string()));
    }

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str
        .trim()
        .parse()
        .map_err(|_| CLIError::GitCommandFailed("Failed to parse commit count".to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

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
