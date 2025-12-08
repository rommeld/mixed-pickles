use std::{io, process::Command};

fn main() -> Result<(), io::Error> {
    let log_command = Command::new("git").arg("log").arg("--oneline").output()?;
    println!("{:?}", log_command);
    Ok(())
}
