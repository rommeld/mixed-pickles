use std::process::Command;

/// Helper to run the binary and capture output
fn run_binary() -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--quiet"])
        .output()
        .expect("Failed to execute command")
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

    // The output should either be empty (all commits have subjects > 10 chars)
    // or contain warning messages about short subjects
    if !stdout.is_empty() {
        assert!(
            stdout.contains("has less than 10 characters"),
            "Output should contain the expected warning format"
        );
    }
}

#[test]
fn output_format_contains_hash_prefix() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // If there are warnings, they should start with "Hash"
    for line in stdout.lines() {
        if !line.is_empty() {
            assert!(
                line.starts_with("Hash"),
                "Warning lines should start with 'Hash', got: {}",
                line
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
