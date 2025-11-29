use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub subject: String,
}

fn fetch_log() -> io::Result<String> {
    // Check if .git directory exists before attempting to run git log
    get_git_dir()?;

    // Run `git log` to fetch all commits
    let log = Command::new("git")
        .arg("log")
        .arg("--pretty=format:'%H|%an|%ae|%s'")
        .output()?;

    if log.status.success() {
        Ok(String::from_utf8_lossy(&log.stdout).to_string())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&log.stderr).to_string(),
        ))
    }
}

fn get_git_dir() -> io::Result<()> {
    if Path::new(".git").exists() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            ".git directory not found",
        ))
    }
}

pub fn parse_commit() -> Option<Commit> {
    let log = fetch_log().ok()?;
    let first_line = log.lines().next()?;

    // Remove surrounding quotes from the format string
    let line = first_line.trim_matches('\'');

    let parts: Vec<&str> = line.splitn(4, '|').collect();
    if parts.len() == 4 {
        Some(Commit {
            hash: parts[0].to_string(),
            author_name: parts[1].to_string(),
            author_email: parts[2].to_string(),
            subject: parts[3].to_string(),
        })
    } else {
        None
    }
}
