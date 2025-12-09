use std::{io, process::Command};

#[derive(Debug)]
#[allow(dead_code)]
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

    println!("Analyzed {:?} commits", &commit_vec.len());
    println!("Found {:?} commits with short messages (< 10 characters)", &improved_hash_output.len());
    println!("{:?}: {:?}", &improved_hash_output, &improved_subject_output);

    Ok(())
}
