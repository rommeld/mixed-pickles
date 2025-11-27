use std::io;
use std::path::Path;
use std::process::Command;

pub fn fetch_log() -> io::Result<String> {
    // Check if .git directory exists before attempting to run git log
    get_git_dir()?;

    // Run `git log` to fetch all commits
    let log = Command::new("git").arg("log").arg("--pretty=format:'%H|%an|%ae|%s'").output()?;

    if log.status.success() {
        Ok(String::from_utf8_lossy(&log.stdout).to_string())
    } else {
        Err(io::Error::other(
            // TODO: Use `stderr` for managing error messages
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

// TODO: Add tests
