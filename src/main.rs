use clap::Parser;
use std::{env, io, path::PathBuf, process::Command};

#[derive(Parser, Debug)]
struct GitCLI {
    // --path /path/to/repo
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(short, long)]
    limit: Option<usize>,
    #[arg(short, long, default_value_t = 30)]
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

#[allow(dead_code)]
fn fetch_current_path() -> Result<PathBuf, io::Error> {
    env::current_dir()
}

fn main() -> Result<(), io::Error> {
    let git_cli = GitCLI::parse();

    let args: Vec<String> = vec![
        "log".to_string(),
        "--pretty=format:%H|%an|%ae|%s".to_string(),
    ];

    let mut command = Command::new("git");

    if let Some(path) = &git_cli.path {
        command.current_dir(path);
    }

    let log_command = command.args(&args).output()?;

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
    let commits_to_check: Box<dyn Iterator<Item = &Commit>> = match git_cli.limit {
        Some(limit) => Box::new(commit_vec.iter().take(limit)),
        None => Box::new(commit_vec.iter()),
    };

    for v in commits_to_check {
        // TODO: Handle without suffix 'feat', 'fix', 'refact', and 'doc'
        if v.subject.len() <= git_cli.threshold {
            improved_hash_output.push(&v.hash[..7]);
            improved_subject_output.push(&v.subject);
        }
    }

    if improved_hash_output.is_empty() {
        println!("Commit messages are adequately executed.");
    } else {
        let analyzed_count = git_cli.limit.unwrap_or(commit_vec.len());
        println!("Analyzed {} commits\n", analyzed_count);
        println!(
            "Found {} commits with short messages (< {} chars):",
            improved_hash_output.len(),
            &git_cli.threshold
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
