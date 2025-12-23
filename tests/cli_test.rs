mod common;

use common::{run_binary, run_binary_with_args};

#[test]
fn no_arguments_uses_defaults() {
    let output = run_binary();
    assert!(
        output.status.success(),
        "Should succeed with default arguments"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") || stdout.contains("adequately executed"),
        "Should produce valid output, got: {}",
        stdout
    );
}

#[test]
fn limit_flag_restricts_commits_analyzed() {
    let output = run_binary_with_args(&["--limit", "5"]);
    assert!(output.status.success(), "Should succeed with --limit flag");
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Analyzed") {
        assert!(
            stdout.contains("Analyzed 5 of"),
            "Should analyze exactly 5 commits, got: {}",
            stdout
        );
    }
}

#[test]
fn threshold_flag_changes_character_limit() {
    let output = run_binary_with_args(&["--threshold", "50"]);
    assert!(
        output.status.success(),
        "Should succeed with --threshold flag"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("short messages") {
        assert!(
            stdout.contains("< 50 chars"),
            "Should show threshold of 50, got: {}",
            stdout
        );
    }
}

#[test]
fn both_flags_together() {
    let output = run_binary_with_args(&["--limit", "10", "--threshold", "25"]);
    assert!(output.status.success(), "Should succeed with both flags");
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Analyzed") {
        assert!(
            stdout.contains("Analyzed 10 of"),
            "Should analyze 10 commits, got: {}",
            stdout
        );
    }
    if stdout.contains("short messages") {
        assert!(
            stdout.contains("< 25 chars"),
            "Should show threshold of 25, got: {}",
            stdout
        );
    }
}

#[test]
fn short_limit_flag() {
    let output = run_binary_with_args(&["-l", "3"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Should succeed with -l short flag, stderr: {}",
        stderr
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed 3 of") || stdout.contains("adequately executed"),
        "Should analyze 3 commits or show success message, got: {}",
        stdout
    );
}

#[test]
fn short_threshold_flag() {
    let output = run_binary_with_args(&["-t", "40"]);
    assert!(output.status.success(), "Should succeed with -t short flag");
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("short messages") {
        assert!(
            stdout.contains("< 40 chars"),
            "Should show threshold of 40 with short flag, got: {}",
            stdout
        );
    }
}

#[test]
fn combined_short_flags() {
    let output = run_binary_with_args(&["-l", "5", "-t", "20"]);
    assert!(
        output.status.success(),
        "Should succeed with combined short flags"
    );
}

#[test]
fn path_flag_with_valid_repo() {
    let output = run_binary_with_args(&["--path", "."]);
    assert!(
        output.status.success(),
        "Should succeed with path to current repo"
    );
}

#[test]
fn path_with_other_flags() {
    let output = run_binary_with_args(&["--path", ".", "-l", "5", "-t", "35"]);
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
        stdout.contains("--limit") && stdout.contains("--threshold") && stdout.contains("--path"),
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
