use clap::Parser;

use mixed_pickles::{GitCLI, error::CLIError};

fn main() {
    let cli = GitCLI::parse();
    if let Err(e) = cli.run() {
        match e {
            CLIError::ValidationFailed(_) => {}
            _ => eprintln!("Error: {}", e),
        }
        std::process::exit(1);
    }
}
