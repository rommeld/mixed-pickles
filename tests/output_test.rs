//! Tests for output formatting

mod common;

use common::{run_binary, run_binary_with_args};

#[test]
fn binary_runs_successfully() {
    let output = run_binary();
    assert!(
        output.status.success(),
        "Binary should exit with success status"
    );
}

#[test]
fn binary_produces_valid_output() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stdout.is_empty() {
        let has_success_msg = stdout.contains("adequately executed");
        let has_warning_msg = stdout.contains("short messages");
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

    if stdout.contains("short messages") {
        let has_indented_commit = stdout.lines().any(|line| line.starts_with("  "));
        assert!(
            has_indented_commit,
            "Short commits should be listed with indentation, got: {}",
            stdout
        );
    }
}

#[test]
fn acceptable_status_shows_success_message() {
    // With a very low threshold, all commits should pass
    let output = run_binary_with_args(&["-t", "0"]);
    assert!(output.status.success());
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
    let output = run_binary_with_args(&["-t", "100"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("short messages") {
        assert!(
            stdout.contains("Analyzed") && stdout.contains("total commits"),
            "Should show analysis header, got: {}",
            stdout
        );
        assert!(
            stdout.contains("Found") && stdout.contains("commits with short messages"),
            "Should show count of short commits, got: {}",
            stdout
        );
    }
}

#[test]
fn needs_work_status_shows_commit_list() {
    let output = run_binary_with_args(&["-t", "100"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("short messages") {
        let has_commit_format = stdout
            .lines()
            .any(|line| line.starts_with("  ") && line.contains(": \"") && line.ends_with("\""));
        assert!(
            has_commit_format,
            "Should list commits in format '  hash: \"subject\"', got: {}",
            stdout
        );
    }
}

#[test]
fn output_shows_threshold_value() {
    let output = run_binary_with_args(&["-t", "75"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("short messages") {
        assert!(
            stdout.contains("< 75 chars"),
            "Should show the threshold value in output, got: {}",
            stdout
        );
    }
}

#[test]
fn output_shows_path_info() {
    let output = run_binary_with_args(&["--path", ".", "-t", "100"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Analyzed") {
        assert!(
            stdout.contains("on path"),
            "Should show path info in output, got: {}",
            stdout
        );
    }
}

#[test]
fn output_shows_analyzed_vs_total_commits() {
    let output = run_binary_with_args(&["-l", "5", "-t", "100"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Analyzed") {
        assert!(
            stdout.contains("5 of") && stdout.contains("total commits"),
            "Should show 'X of Y total commits', got: {}",
            stdout
        );
    }
}

#[test]
fn commit_hash_is_seven_characters() {
    let output = run_binary_with_args(&["-t", "100"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.starts_with("  ") && line.contains(": \"") {
            // Extract hash (format: "  abc1234: \"subject\"")
            let hash = line.trim().split(':').next().unwrap();
            assert_eq!(
                hash.len(),
                7,
                "Hash should be 7 characters, got: {} ({})",
                hash,
                hash.len()
            );
        }
    }
}
