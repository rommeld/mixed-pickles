mod commit;
mod error;

use clap::Parser;
use std::path::PathBuf;

use commit::{Commit, print_results, validate_repo_path};
use error::CLIError;

#[derive(Parser, Debug)]
struct GitCLI {
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(short, long)]
    limit: Option<usize>,
    #[arg(short, long, default_value_t = 30)]
    threshold: usize,
    // TODO: Add a new argument to specify release type (release)
}

fn main() -> Result<(), CLIError> {
    let git_cli = GitCLI::parse();

    if let Some(path) = &git_cli.path {
        validate_repo_path(path)?;
    }

    let commits = Commit::fetch_all(git_cli.path.as_ref(), git_cli.limit)?;

    let short_commits = Commit::find_short(&commits, git_cli.threshold);

    let analyzed_count = commits.len();
    print_results(
        &short_commits,
        commits.len(),
        analyzed_count,
        git_cli.threshold,
        &git_cli.path,
    );

    Ok(())
}
