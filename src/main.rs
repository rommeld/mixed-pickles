use clap::Parser;
use std::{io, process::Command};
use thiserror::Error;

#[derive(Parser)]
struct GitCLI {
    #[arg(long)]
    repo_path: String,
    #[arg(short, long)]
    limit: usize,
    #[arg(short, long)]
    threshold: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Commit {
    hash: String,
    author_name: String,
    // TODO: validate email with '@'
    author_email: String,
    subject: String,
}

#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("Failed to execute bianry.")]
    ExecutionError(#[from] io::Error),
    #[error("{path} not a git repository.")]
    RepositoryError { path: String },
    #[error("Output is not a valid UTF-8.")]
    UTFError(String),
}

fn main() -> Result<(), crate::AnalyzerError> {
    let log_command = Command::new("git")
        .arg("log")
        .arg("--pretty=format:%H|%an|%ae|%s")
        .args(["-n", "5"])
        .output()?;

    let log_output = String::from_utf8_lossy(&log_command.stdout);

    let parsed_commit: Vec<&str> = log_output.lines().collect();

    let mut commit_vec = Vec::new();

    for commit_message in parsed_commit
        .into_iter()
        .map(|pipe_character| pipe_character.split("|"))
    {
        let mut log_vector = Vec::new();

        for i in commit_message {
            log_vector.push(i)
        }

        if log_vector.len() == 4 {
            let commit = Commit {
                hash: log_vector[0].to_string(),
                author_name: log_vector[1].to_string(),
                author_email: log_vector[2].to_string(),
                subject: log_vector[3].to_string(),
            };
            commit_vec.push(commit);
        } else {
            println!("Log couldn't be loaded.")
        };
    }

    let mut improved_hash_output = Vec::new();
    let mut improved_subject_output = Vec::new();
    // Check for subject length
    for v in &commit_vec {
        // TODO: Handle without suffix 'feat', 'fix', 'refact', and 'doc'
        if v.subject.len() <= 10 {
            improved_hash_output.push(&v.hash[..7]);
            improved_subject_output.push(&v.subject);
        }
    }

    if improved_hash_output.is_empty() {
        println!("Commit messages are adequately executed.");
    } else {
        println!("Analyzed {} commits\n", commit_vec.len());
        println!(
            "Found {} commits with short messages (< 10 chars):",
            improved_hash_output.len()
        );
        for (hash, subject) in improved_hash_output
            .iter()
            .zip(improved_subject_output.iter())
        {
            println!("  {}: \"{}\"", hash, subject);
        }
    }

    Ok(())
}
