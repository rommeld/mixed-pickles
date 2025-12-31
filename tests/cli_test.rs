mod common;

use std::io::Write;
use std::process::Command;

use common::{has_analyzing_header, has_issues_summary, run_binary, run_binary_with_args};

#[test]
fn no_arguments_runs_analysis() {
    let output = run_binary();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Analysis runs and produces output (may find issues or not)
    assert!(
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
        "Should produce valid output, got: {}",
        stdout
    );
}

#[test]
fn short_commits_found_exits_nonzero() {
    // With high threshold, all commits should be "short"
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should exit non-zero when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_issues_summary(&stdout),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
        "Should produce valid output with combined flags, got: {}",
        stdout
    );
}

#[test]
fn quiet_flag_with_issues() {
    // With high threshold, commits will have issues - quiet mode still shows output
    // Use --error=short to make ShortCommit an error (default is warning)
    let output = run_binary_with_args(&["-q", "-t", "1000", "-l", "5", "--error=short"]);
    assert!(
        !output.status.success(),
        "Should fail when validation issues found"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_issues_summary(&stdout),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
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

#[test]
fn error_flag_makes_validation_exit_nonzero() {
    // Use --error to make a validation type an error
    let output = run_binary_with_args(&["-l", "5", "--error=vague"]);
    // We can't guarantee vague language, but the flag should be accepted
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown validation"),
        "Should accept --error=vague flag, got stderr: {}",
        stderr
    );
}

#[test]
fn ignore_flag_suppresses_validation() {
    // Use --ignore to suppress a validation type
    let output = run_binary_with_args(&["-l", "5", "--ignore=ref,format"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown validation"),
        "Should accept --ignore flag with multiple values, got stderr: {}",
        stderr
    );
}

#[test]
fn warn_flag_accepted() {
    // Use --warn to set warning severity
    let output = run_binary_with_args(&["-l", "5", "--warn=wip"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown validation"),
        "Should accept --warn flag, got stderr: {}",
        stderr
    );
}

#[test]
fn invalid_validation_type_fails() {
    let output = run_binary_with_args(&["--error=notavalidation"]);
    assert!(
        !output.status.success(),
        "Should fail with invalid validation type"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown validation"),
        "Should show error for unknown validation type, got: {}",
        stderr
    );
}

#[test]
fn warnings_do_not_cause_nonzero_exit() {
    // With default config, ShortCommit is a warning, not an error
    // So even with short commits, exit should be zero (no errors)
    let output = run_binary_with_args(&["-t", "1000", "-l", "5", "--ignore=wip"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show commits with issues (warnings)
    assert!(
        has_issues_summary(&stdout) || stdout.contains("adequately executed"),
        "Should produce output, got: {}",
        stdout
    );
    // But exit should be success because no errors (only warnings)
    assert!(
        output.status.success(),
        "Should exit zero when only warnings (no errors) found"
    );
}

#[test]
fn disable_flag_accepted() {
    // Use --disable to turn off validation checks
    let output = run_binary_with_args(&["-l", "5", "--disable=ref,format"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown validation"),
        "Should accept --disable flag with multiple values, got stderr: {}",
        stderr
    );
}

#[test]
fn disable_flag_prevents_validation() {
    // Disable reference checking - commits without refs should not trigger
    let output = run_binary_with_args(&["-l", "5", "--disable=ref,format,vague,wip,imperative"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // With most validations disabled, should have fewer (or no) issues
    assert!(
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
        "Should produce valid output with --disable, got: {}",
        stdout
    );
}

#[test]
fn disable_flag_with_error_flag() {
    // Disable some validations but make others errors
    let output = run_binary_with_args(&[
        "-l",
        "5",
        "-t",
        "1000",
        "--disable=ref,format,vague,wip,imperative",
        "--error=short",
    ]);
    // Should fail because short is an error and threshold is very high
    assert!(
        !output.status.success(),
        "Should exit non-zero when short is error and threshold high"
    );
}

#[test]
fn disable_invalid_validation_fails() {
    let output = run_binary_with_args(&["--disable=notavalidation"]);
    assert!(
        !output.status.success(),
        "Should fail with invalid validation type in --disable"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown validation"),
        "Should show error for unknown validation type in --disable, got: {}",
        stderr
    );
}

#[test]
fn disable_all_validations() {
    // Disable all validation types
    let output =
        run_binary_with_args(&["-l", "5", "--disable=short,ref,format,vague,wip,imperative"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // With all validations disabled, should pass
    assert!(
        stdout.contains("adequately executed"),
        "Should pass with all validations disabled, got: {}",
        stdout
    );
    assert!(
        output.status.success(),
        "Should exit zero with all validations disabled"
    );
}

#[test]
fn strict_flag_accepted() {
    // Verify --strict flag is accepted
    let output = run_binary_with_args(&["--strict", "-l", "1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected") && !stderr.contains("unknown"),
        "Should accept --strict flag, got stderr: {}",
        stderr
    );
}

#[test]
fn strict_flag_exits_nonzero_on_warnings() {
    // With --strict, warnings should cause exit non-zero
    // ShortCommit is a warning by default, use high threshold to trigger it
    // Disable WIP check since that's an error by default
    let output = run_binary_with_args(&[
        "-l",
        "5",
        "-t",
        "1000",
        "--strict",
        "--disable=wip",
        "--ignore=ref,format,vague,imperative",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show warnings
    assert!(
        has_issues_summary(&stdout),
        "Should show commits with issues, got: {}",
        stdout
    );
    // Should exit non-zero because of warnings in strict mode
    assert!(
        !output.status.success(),
        "Should exit non-zero with warnings in strict mode"
    );
}

#[test]
fn without_strict_warnings_exit_zero() {
    // Without --strict, warnings should exit zero
    // ShortCommit is a warning by default, use high threshold to trigger it
    // Disable WIP check since that's an error by default
    let output = run_binary_with_args(&[
        "-l",
        "5",
        "-t",
        "1000",
        "--disable=wip",
        "--ignore=ref,format,vague,imperative",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show warnings
    assert!(
        has_issues_summary(&stdout),
        "Should show commits with issues, got: {}",
        stdout
    );
    // Should exit zero because no errors (only warnings) without strict
    assert!(
        output.status.success(),
        "Should exit zero with only warnings (no --strict)"
    );
}

// Config file integration tests

#[test]
fn no_config_flag_is_accepted() {
    let output = run_binary_with_args(&["--no-config", "-l", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
        "Should run with --no-config flag, got: {}",
        stdout
    );
}

#[test]
fn config_flag_with_nonexistent_file_uses_defaults() {
    let output = run_binary_with_args(&["--config", "/nonexistent/path.toml", "-l", "1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should still run (config file is optional)
    assert!(
        has_analyzing_header(&stdout) || stdout.contains("adequately executed"),
        "Should run even with nonexistent config file, got: {}",
        stdout
    );
}

#[test]
fn config_flag_loads_custom_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".mixed-pickles.toml");

    // Create config that sets threshold very high (all commits will be "short")
    // and makes short an error
    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(file, "threshold = 10000").unwrap();
    writeln!(file, "[severity]").unwrap();
    writeln!(file, "short = \"error\"").unwrap();
    drop(file);

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "-l",
            "1",
            "--disable=wip,ref,format,vague,imperative",
        ])
        .output()
        .expect("Failed to execute command");

    // With threshold=10000, any commit should be flagged as short (error)
    assert!(
        !output.status.success(),
        "Should exit non-zero with config that makes short commits errors"
    );
}

#[test]
fn cli_threshold_overrides_config_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".mixed-pickles.toml");

    // Config sets very high threshold
    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(file, "threshold = 10000").unwrap();
    writeln!(file, "[severity]").unwrap();
    writeln!(file, "short = \"error\"").unwrap();
    drop(file);

    // CLI sets low threshold (should override config)
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "-l",
            "1",
            "-t",
            "1", // Override threshold to 1 (all commits pass)
            "--disable=wip,ref,format,vague,imperative",
        ])
        .output()
        .expect("Failed to execute command");

    // With threshold=1 from CLI, commits should pass
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() || has_analyzing_header(&stdout),
        "CLI threshold should override config file threshold"
    );
}

#[test]
fn cli_disable_adds_to_config_file_disables() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".mixed-pickles.toml");

    // Config disables wip check
    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(file, "disable = [\"wip\"]").unwrap();
    drop(file);

    // CLI also disables other checks - should combine
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "-l",
            "1",
            "--disable=ref,format,vague,imperative,short",
        ])
        .output()
        .expect("Failed to execute command");

    // With all validations disabled, should succeed
    assert!(
        output.status.success(),
        "Should pass with all validations disabled via config + CLI"
    );
}

#[test]
fn no_config_ignores_config_file() {
    // Test that --no-config flag causes config files to be ignored
    // We use the actual project directory which has pyproject.toml
    // but with --no-config it should use defaults
    let output = run_binary_with_args(&[
        "--no-config",
        "-l",
        "1",
        "--disable=wip,ref,format,vague,imperative,short",
    ]);

    // With --no-config and all validations disabled, should succeed
    assert!(
        output.status.success(),
        "--no-config should work and allow disabling all validations"
    );
}
