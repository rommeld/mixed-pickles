use std::process::Command;

pub fn run_binary_with_args(args: &[&str]) -> std::process::Output {
    let mut cmd_args = vec!["run", "--quiet", "--"];
    cmd_args.extend(args);
    Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute command")
}

pub fn run_binary() -> std::process::Output {
    run_binary_with_args(&[])
}

/// Check if output contains "commit(s) with issues" (handles singular/plural).
pub fn has_issues_summary(stdout: &str) -> bool {
    stdout.contains("commit with issues") || stdout.contains("commits with issues")
}

/// Check if output contains the analysis header (handles singular/plural).
pub fn has_analyzing_header(stdout: &str) -> bool {
    stdout.contains("Analyzed")
        && (stdout.contains("commit of") || stdout.contains("commits of"))
        && stdout.contains("total in")
}
