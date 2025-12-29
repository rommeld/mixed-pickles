mod common;

use common::{run_binary, run_binary_with_args};

#[test]
fn no_arguments_runs_analysis() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // With default threshold, we expect short commits to be found (exit code 1)
    // or no short commits (exit code 0)
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output, got: {}",
        stdout
    );
}

#[test]
fn no_short_commits_exits_zero() {
    // With threshold=0, no commits should be considered "short"
    let output = run_binary_with_args(&["-t", "0"]);
    assert!(
        output.status.success(),
        "Should exit 0 when no short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("adequately executed"),
        "Should show success message, got: {}",
        stdout
    );
}

#[test]
fn short_commits_found_exits_nonzero() {
    // With high threshold, all commits should be "short"
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("commits with issues"),
        "Should show commits with issues, got: {}",
        stdout
    );
}

#[test]
fn limit_flag_restricts_commits_analyzed() {
    let output = run_binary_with_args(&["--limit", "5", "-t", "0"]);
    assert!(output.status.success(), "Should succeed with --limit flag");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("adequately executed"),
        "Should show success with low threshold, got: {}",
        stdout
    );
}

#[test]
fn threshold_flag_changes_character_limit() {
    let output = run_binary_with_args(&["--threshold", "1000", "-l", "3"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("threshold: 1000 chars"),
        "Should show threshold of 1000, got: {}",
        stdout
    );
}

#[test]
fn both_flags_together() {
    let output = run_binary_with_args(&["--limit", "10", "--threshold", "0"]);
    assert!(
        output.status.success(),
        "Should succeed with both flags when no issues"
    );
}

#[test]
fn short_limit_flag() {
    let output = run_binary_with_args(&["-l", "3", "-t", "0"]);
    assert!(output.status.success(), "Should succeed with -l short flag");
}

#[test]
fn short_threshold_flag() {
    let output = run_binary_with_args(&["-t", "0"]);
    assert!(output.status.success(), "Should succeed with -t short flag");
}

#[test]
fn combined_short_flags() {
    let output = run_binary_with_args(&["-l", "5", "-t", "0"]);
    assert!(
        output.status.success(),
        "Should succeed with combined short flags"
    );
}

#[test]
fn quiet_flag_suppresses_output_on_success() {
    let output = run_binary_with_args(&["-q", "-t", "0"]);
    assert!(output.status.success(), "Should succeed with -q flag");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "Should have no output in quiet mode on success, got: {}",
        stdout
    );
}

#[test]
fn quiet_flag_shows_output_on_failure() {
    let output = run_binary_with_args(&["-q", "-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should fail when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("commits with issues"),
        "Should show output in quiet mode when issues found, got: {}",
        stdout
    );
}

#[test]
fn path_flag_with_valid_repo() {
    let output = run_binary_with_args(&["--path", ".", "-t", "0"]);
    assert!(
        output.status.success(),
        "Should succeed with path to current repo"
    );
}

#[test]
fn path_with_other_flags() {
    let output = run_binary_with_args(&["--path", ".", "-l", "5", "-t", "0"]);
    assert!(
        output.status.success(),
        "Should succeed with path and other flags combined"
    );
}

#[test]
fn non_numeric_limit_fails() {
    let output = run_binary_with_args(&["--limit", "abc"]);
    assert!(
        !output.status.success(),
        "Should fail with non-numeric limit"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("error"),
        "Should show helpful error for non-numeric limit, got: {}",
        stderr
    );
}

#[test]
fn non_numeric_threshold_fails() {
    let output = run_binary_with_args(&["--threshold", "xyz"]);
    assert!(
        !output.status.success(),
        "Should fail with non-numeric threshold"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("error"),
        "Should show helpful error for non-numeric threshold, got: {}",
        stderr
    );
}

#[test]
fn non_existent_path_fails() {
    let output = run_binary_with_args(&["--path", "/this/path/does/not/exist"]);
    assert!(
        !output.status.success(),
        "Should fail with non-existent path"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist") || stderr.contains("PathNotFound"),
        "Should show helpful error for non-existent path, got: {}",
        stderr
    );
}

#[test]
fn path_not_a_repository_fails() {
    let output = run_binary_with_args(&["--path", "/tmp"]);
    assert!(
        !output.status.success(),
        "Should fail when path is not a git repository"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a git repository") || stderr.contains("NotARepository"),
        "Should show helpful error for non-repo path, got: {}",
        stderr
    );
}

#[test]
fn help_flag_shows_usage() {
    let output = run_binary_with_args(&["--help"]);
    assert!(output.status.success(), "Should succeed with --help flag");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--limit")
            && stdout.contains("--threshold")
            && stdout.contains("--path")
            && stdout.contains("--quiet"),
        "Help should show all available flags, got: {}",
        stdout
    );
}

#[test]
fn unknown_flag_fails() {
    let output = run_binary_with_args(&["--unknown-flag"]);
    assert!(!output.status.success(), "Should fail with unknown flag");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected") || stderr.contains("error") || stderr.contains("unknown"),
        "Should show helpful error for unknown flag, got: {}",
        stderr
    );
}
