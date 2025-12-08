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
        .arg("--pretty=format:%H|%an|%ae|%s")
        .args(["-n", "5"])
        .output()?;

    let log_output = String::from_utf8_lossy(&log_command.stdout);

    let log_opt = log_output
        .lines()
        .next()
        .expect("Failed to fetch first log.")
        .split("|");

    let mut log_vector = Vec::new();

    for i in log_opt {
        log_vector.push(i)
    }

    if log_vector.len() == 4 {
        let commit = Commit {
            hash: log_vector[0].to_string(),
            author_name: log_vector[1].to_string(),
            author_email: log_vector[2].to_string(),
            subject: log_vector[3].to_string(),
        };
        println!("{:?}", commit);
    } else {
        println!("Log couldn't be loaded.")
    };

    Ok(())
}
