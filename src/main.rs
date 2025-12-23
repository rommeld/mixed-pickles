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
}

fn main() -> Result<(), CLIError> {
    let git_cli = GitCLI::parse();

    // Validate path if provided
    if let Some(path) = &git_cli.path {
        validate_repo_path(path)?;
    }

    // Fetch commits
    let commits = Commit::fetch_all(git_cli.path.as_ref())?;

    // Find short commits
    let short_commits = Commit::find_short(&commits, git_cli.limit, git_cli.threshold);

    // Print results
    let analyzed_count = git_cli.limit.unwrap_or(commits.len());
    print_results(
        &short_commits,
        commits.len(),
        analyzed_count,
        git_cli.threshold,
        &git_cli.path,
    );

    Ok(())
}
