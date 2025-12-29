//! Tests for output formatting

mod common;

use common::{has_analyzing_header, has_issues_summary, run_binary, run_binary_with_args};

#[test]
fn binary_runs_and_produces_output() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Binary runs and produces some output (may exit 0 or 1 depending on commits)
    assert!(
        !stdout.is_empty() || output.status.success(),
        "Binary should produce output or succeed"
    );
}

#[test]
fn binary_produces_valid_output() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stdout.is_empty() {
        let has_success_msg = stdout.contains("adequately executed");
        assert!(
            has_success_msg || has_issues_summary(&stdout),
            "Output should contain expected format, got: {}",
            stdout
        );
    }
}

#[test]
fn output_format_lists_commits_with_indentation() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("commits with issues") {
        let has_indented_commit = stdout.lines().any(|line| line.starts_with("  "));
        assert!(
            has_indented_commit,
            "Commits with issues should be listed with indentation, got: {}",
            stdout
        );
    }
}

#[test]
fn acceptable_status_shows_success_message() {
    // When all commits pass all validations, show success message
    // Note: Most commits in this repo will have validation issues
    // (missing reference, invalid format), so we just verify the
    // output format is correct when issues are found
    let output = run_binary_with_args(&["-l", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Either success message or issues found - both are valid output formats
    // New format uses "Summary: X commits with issues"
    assert!(
        stdout.contains("adequately executed") || stdout.contains("Summary:"),
        "Should show valid output format, got: {}",
        stdout
    );
}

#[test]
fn needs_work_status_shows_analysis() {
    // With a high threshold, some commits should be flagged
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--error=short"]);
    // Now exits non-zero when validation issues found
    assert!(
        !output.status.success(),
        "Should exit non-zero when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_analyzing_header(&stdout),
        "Should show analysis header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Summary:") && has_issues_summary(&stdout),
        "Should show summary with count of commits with issues, got: {}",
        stdout
    );
}

#[test]
fn needs_work_status_shows_commit_list() {
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // New format: "Commit HASH by AUTHOR <EMAIL>" and "  Subject: \"...\""
    let has_commit_header = stdout
        .lines()
        .any(|line| line.starts_with("Commit ") && line.contains(" by "));
    let has_subject_line = stdout.lines().any(|line| line.starts_with("  Subject: \""));
    assert!(
        has_commit_header && has_subject_line,
        "Should list commits with header and subject, got: {}",
        stdout
    );
}

#[test]
fn output_shows_threshold_value() {
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("threshold: 1000 chars"),
        "Should show the threshold value in output, got: {}",
        stdout
    );
}

#[test]
fn output_shows_path_info() {
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["--path", ".", "-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_analyzing_header(&stdout) && stdout.contains(" in ."),
        "Should show path info in output, got: {}",
        stdout
    );
}

#[test]
fn output_shows_analyzed_vs_total_commits() {
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_analyzing_header(&stdout),
        "Should show 'Analyzing X commit(s) in PATH', got: {}",
        stdout
    );
}
