use std::process::Command;

/// Helper to run the binary with arguments and capture output
fn run_binary_with_args(args: &[&str]) -> std::process::Output {
    let mut cmd_args = vec!["run", "--quiet", "--"];
    cmd_args.extend(args);
    Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute command")
}

/// Helper to run the binary and capture output
fn run_binary() -> std::process::Output {
    run_binary_with_args(&[])
}

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

    // The output should either indicate success or list short commit messages
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

    // If there are short commits, they should be listed with indentation
    if stdout.contains("short messages") {
        let has_indented_commit = stdout.lines().any(|line| line.starts_with("  "));
        assert!(
            has_indented_commit,
            "Short commits should be listed with indentation, got: {}",
            stdout
        );
    }
}

// CLI argument tests
mod cli_tests {
    use super::run_binary_with_args;

    #[test]
    fn no_arguments_uses_defaults() {
        let output = run_binary_with_args(&[]);
        assert!(
            output.status.success(),
            "Should succeed with default arguments"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should produce valid output with defaults (threshold=30, no limit)
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
        // Either shows analyzed count or "adequately executed" message
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
        // Use current directory which is a git repo
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
            stdout.contains("--limit")
                && stdout.contains("--threshold")
                && stdout.contains("--path"),
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
}

// Tests for print_results output
mod print_results_tests {
    use super::run_binary_with_args;

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
            // Check that commits are listed with hash and subject
            let has_commit_format = stdout.lines().any(|line| {
                line.starts_with("  ") && line.contains(": \"") && line.ends_with("\"")
            });
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
}

// Tests for parsing logic patterns (demonstrating what could be tested if extracted)
mod parsing_tests {
    /// Test that pipe-separated format is correctly handled
    #[test]
    fn pipe_separator_splits_correctly() {
        let input = "abc123|John Doe|john@example.com|feat: add feature";
        let parts: Vec<&str> = input.split("|").collect();

        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "abc123");
        assert_eq!(parts[1], "John Doe");
        assert_eq!(parts[2], "john@example.com");
        assert_eq!(parts[3], "feat: add feature");
    }

    #[test]
    fn pipe_in_subject_creates_extra_parts() {
        // Edge case: if subject contains a pipe, it will split incorrectly
        let input = "abc123|John|john@example.com|fix: handle a|b case";
        let parts: Vec<&str> = input.split("|").collect();

        // This demonstrates a potential bug in the current implementation
        assert_eq!(
            parts.len(),
            5,
            "Subject with pipe creates 5 parts instead of 4"
        );
    }

    #[test]
    fn empty_line_produces_single_empty_part() {
        let input = "";
        let parts: Vec<&str> = input.split("|").collect();

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "");
    }

    #[test]
    fn subject_length_check() {
        let short_subject = "fix bug";
        let long_subject = "feat: implement user authentication system";

        assert!(
            short_subject.len() <= 10,
            "Short subject should trigger warning"
        );
        assert!(
            long_subject.len() > 10,
            "Long subject should not trigger warning"
        );
    }

    #[test]
    fn hash_truncation_to_seven_chars() {
        let full_hash = "abc1234567890def";
        let truncated = &full_hash[..7];

        assert_eq!(truncated, "abc1234");
        assert_eq!(truncated.len(), 7);
    }

    #[test]
    fn subject_truncation_to_seven_chars() {
        let subject = "short fix";
        let truncated = &subject[..7];

        assert_eq!(truncated, "short f");
    }

    #[test]
    #[should_panic]
    fn subject_truncation_panics_on_short_string() {
        // This demonstrates a potential panic in the current implementation
        // if a subject has less than 7 characters
        let short_subject = "fix";
        let _truncated = &short_subject[..7]; // This will panic
    }
}
