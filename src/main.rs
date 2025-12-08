use std::{io, process::Command};

#[derive(Debug)]
struct Commit {
    hash: String,
    author_name: String,
    // TODO: validate email with '@'
    author_email: String,
    subject: String,
}

fn main() -> Result<(), io::Error> {
    let log_command = Command::new("git")
        .arg("log")
        .arg("--pretty=format:'%H|%an|%ae|%s'")
        .args(["-n", "5"])
        .output()?;
    
    let log_output = String::from_utf8_lossy(&log_command.stdout).to_string(); 
    
    let mut log_opt = log_output.lines().next().expect("Failed to fetch first log.").split("|");

    let commit = Commit {
        hash: log_opt.next().expect("Failed to get hash.").to_string(),
        author_name: log_opt.next().expect("Failed to get hash.").to_string(),
        author_email: log_opt.next().expect("Failed to get hash.").to_string(),
        subject: log_opt.next().expect("Failed to get hash.").to_string(),
    };

    println!("{:#?}", commit);
    
    Ok(())
}
