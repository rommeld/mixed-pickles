//! Tests for output formatting

mod common;

use common::{run_binary, run_binary_with_args};

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
        let has_warning_msg = stdout.contains("commits with issues");
        assert!(
            has_success_msg || has_warning_msg,
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
    // With a very low threshold, all commits should pass
    let output = run_binary_with_args(&["-t", "0"]);
    assert!(
        output.status.success(),
        "Should exit 0 when no short commits"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("adequately executed"),
        "Should show success message when no short commits found, got: {}",
        stdout
    );
}

#[test]
fn needs_work_status_shows_analysis() {
    // With a high threshold, some commits should be flagged
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
    // Now exits non-zero when validation issues found
    assert!(
        !output.status.success(),
        "Should exit non-zero when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Analyzed") && stdout.contains("total commits"),
        "Should show analysis header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("Found") && stdout.contains("commits with issues"),
        "Should show count of commits with issues, got: {}",
        stdout
    );
}

#[test]
fn needs_work_status_shows_commit_list() {
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_commit_format = stdout
        .lines()
        .any(|line| line.starts_with("  ") && line.contains(": \"") && line.ends_with('"'));
    assert!(
        has_commit_format,
        "Should list commits in format '  hash: \"subject\"', got: {}",
        stdout
    );
}

#[test]
fn output_shows_threshold_value() {
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
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
    let output = run_binary_with_args(&["--path", ".", "-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("on path"),
        "Should show path info in output, got: {}",
        stdout
    );
}

#[test]
fn output_shows_analyzed_vs_total_commits() {
    let output = run_binary_with_args(&["-t", "1000", "-l", "5"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when short commits found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for pattern "Analyzed N of N total commits" (N can be any number)
    let has_format = stdout.contains(" of ") && stdout.contains("total commits");
    assert!(
        has_format,
        "Should show 'X of Y total commits', got: {}",
        stdout
    );
}
