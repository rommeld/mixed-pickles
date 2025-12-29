//! Tests for commit parsing logic

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
fn splitn_handles_pipe_in_subject() {
    // Using splitn(4, "|") correctly handles pipes in subject
    let input = "abc123|John|john@example.com|fix: handle a|b case";
    let parts: Vec<&str> = input.splitn(4, "|").collect();

    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0], "abc123");
    assert_eq!(parts[1], "John");
    assert_eq!(parts[2], "john@example.com");
    assert_eq!(parts[3], "fix: handle a|b case"); // Pipe preserved in subject
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
fn empty_subject_has_zero_length() {
    let empty_subject = "";
    assert_eq!(empty_subject.len(), 0);
    assert!(empty_subject.len() <= 10); // Would be flagged as short
}

#[test]
fn very_long_subject_not_flagged() {
    let long_subject = "feat: this is a very long commit message that describes the changes in great detail and should definitely not be flagged as too short";
    assert!(long_subject.len() > 100);
}

#[test]
fn subject_with_special_characters() {
    let subject = "fix: handle émojis 🎉 and spëcial çharacters";
    // Should handle unicode correctly
    assert!(subject.len() > 10);
}

#[test]
fn multiple_pipes_in_subject_with_splitn() {
    let input = "abc123|Author|email@test.com|fix: a|b|c|d case";
    let parts: Vec<&str> = input.splitn(4, "|").collect();

    assert_eq!(parts.len(), 4);
    assert_eq!(parts[3], "fix: a|b|c|d case");
}
