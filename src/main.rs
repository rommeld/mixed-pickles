use std::{io, process::Command};

fn main() -> Result<(), io::Error> {
    let log_command = Command::new("git").arg("log").args(["-n","5"]).output()?;
    println!("{:?}", String::from_utf8_lossy(&log_command.stdout));
    Ok(())
}
