use mixed_pickles::{parse_commit, Commit};

#[test]
fn test_parse_commit_returns_valid_commit() {
    // This test assumes it's run from a git repository with at least one commit
    let result = parse_commit();

    assert!(result.is_some(), "Expected a commit to be parsed from git log");

    let commit = result.expect("Failed to get commit");

    // Verify all fields are non-empty
    assert!(!commit.hash.is_empty(), "Commit hash should not be empty");
    assert!(
        !commit.author_name.is_empty(),
        "Author name should not be empty"
    );
    assert!(
        !commit.author_email.is_empty(),
        "Author email should not be empty"
    );
    assert!(
        !commit.subject.is_empty(),
        "Commit subject should not be empty"
    );

    // Verify hash is a valid git hash (40 hex characters)
    assert_eq!(commit.hash.len(), 40, "Git hash should be 40 characters");
    assert!(
        commit.hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Git hash should only contain hex characters"
    );

    // Verify email contains @
    assert!(
        commit.author_email.contains('@'),
        "Author email should contain @"
    );
}

#[test]
fn test_commit_struct_debug() {
    let commit = Commit {
        hash: "abc123".to_string(),
        author_name: "Test User".to_string(),
        author_email: "test@example.com".to_string(),
        subject: "Test commit".to_string(),
    };

    let debug_output = format!("{:?}", commit);
    assert!(debug_output.contains("abc123"));
    assert!(debug_output.contains("Test User"));
    assert!(debug_output.contains("test@example.com"));
    assert!(debug_output.contains("Test commit"));
}
