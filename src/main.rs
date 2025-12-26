use clap::Parser;
use std::path::PathBuf;

use mixed_pickles::{commit_analyzer, error::CLIError};

#[derive(Parser, Debug)]
struct GitCLI {
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(short, long)]
    limit: Option<usize>,
    #[arg(short, long, default_value_t = 30)]
    threshold: usize,
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<(), CLIError> {
    let git_cli = GitCLI::parse();

    commit_analyzer(
        git_cli.path.as_ref(),
        git_cli.limit,
        git_cli.threshold,
        git_cli.quiet,
    )?;

    Ok(())
}
