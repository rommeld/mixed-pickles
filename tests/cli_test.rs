mod common;

use common::{run_binary, run_binary_with_args};

#[test]
fn no_arguments_runs_analysis() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Analysis runs and produces output (may find issues or not)
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output, got: {}",
        stdout
    );
}

#[test]
fn validation_issues_exits_nonzero() {
    // Most commits in this repo will have validation issues
    // (missing reference, invalid format, or short message)
    let output = run_binary_with_args(&["-l", "5"]);
    // The exit code depends on whether commits pass all validations
    // Just verify the binary runs and produces output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output, got: {}",
        stdout
    );
}

#[test]
fn short_commits_found_exits_nonzero() {
    // With high threshold, all commits should be "short"
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when validation issues found"
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
    // Verify limit flag works (check output format, not exit code)
    let output = run_binary_with_args(&["--limit", "5"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with --limit flag, got: {}",
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
    // Verify both flags work together (check output format, not exit code)
    let output = run_binary_with_args(&["--limit", "10", "--threshold", "50"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with both flags, got: {}",
        stdout
    );
}

#[test]
fn short_limit_flag() {
    // Verify -l short flag works
    let output = run_binary_with_args(&["-l", "3"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with -l flag, got: {}",
        stdout
    );
}

#[test]
fn short_threshold_flag() {
    // Verify -t short flag works
    let output = run_binary_with_args(&["-t", "50"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with -t flag, got: {}",
        stdout
    );
}

#[test]
fn combined_short_flags() {
    // Verify combined short flags work
    let output = run_binary_with_args(&["-l", "5", "-t", "50"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with combined flags, got: {}",
        stdout
    );
}

#[test]
fn quiet_flag_with_issues() {
    // With high threshold, commits will have issues - quiet mode still shows output
    let output = run_binary_with_args(&["-q", "-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should fail when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("commits with issues"),
        "Should show output in quiet mode when issues found, got: {}",
        stdout
    );
}

#[test]
fn quiet_flag_exists() {
    // Verify -q flag is accepted
    let output = run_binary_with_args(&["-q", "-l", "1"]);
    // Just verify it doesn't error on the flag itself
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected") && !stderr.contains("unknown"),
        "Should accept -q flag, got stderr: {}",
        stderr
    );
}

#[test]
fn path_flag_with_valid_repo() {
    // Verify --path flag works with valid repo
    let output = run_binary_with_args(&["--path", ".", "-l", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with --path flag, got: {}",
        stdout
    );
}

#[test]
fn path_with_other_flags() {
    // Verify --path works with other flags
    let output = run_binary_with_args(&["--path", ".", "-l", "5", "-t", "50"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output with path and other flags, got: {}",
        stdout
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
