use clap::Parser;

use mixed_pickles::{GitCLI, error::CLIError};

fn main() -> Result<(), CLIError> {
    let cli = GitCLI::parse();
    cli.run()
}
