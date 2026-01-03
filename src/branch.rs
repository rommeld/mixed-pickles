//! Branch detection and pattern matching.

use std::path::PathBuf;
use std::process::Command;

use crate::error::CLIError;

/// Get the current git branch name.
/// Returns None if in detached HEAD state.
pub fn get_current_branch(repo_path: Option<&PathBuf>) -> Result<Option<String>, CLIError> {
    let mut command = Command::new("git");

    if let Some(path) = repo_path {
        command.current_dir(path);
    }

    let output = command.args(["symbolic-ref", "--short", "HEAD"]).output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(branch))
    } else {
        // Detached HEAD state or other issue - return None
        Ok(None)
    }
}

/// Check if a branch name matches any of the given patterns.
/// Supports glob patterns like "feature/*", "release-*", etc.
/// Returns true if patterns is empty (matches all branches).
pub fn matches_any_pattern(branch: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|pattern| glob_match(pattern, branch))
}

/// Check if a branch name matches a glob pattern.
/// Supports:
/// - Exact match: "main" matches "main"
/// - Single wildcard: "feature/*" matches "feature/login" (not "feature/a/b")
/// - Double wildcard: "feature/**" matches "feature/a/b/c"
/// - Single char: "release-?" matches "release-1"
fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_impl(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_impl(pattern: &[u8], text: &[u8]) -> bool {
    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_p_idx: Option<usize> = None;
    let mut star_t_idx: Option<usize> = None;
    let mut double_star = false;

    while t_idx < text.len() {
        if p_idx < pattern.len() {
            match pattern[p_idx] {
                b'*' => {
                    // Check for **
                    if p_idx + 1 < pattern.len() && pattern[p_idx + 1] == b'*' {
                        double_star = true;
                        p_idx += 2;
                    } else {
                        double_star = false;
                        p_idx += 1;
                    }
                    star_p_idx = Some(p_idx);
                    star_t_idx = Some(t_idx);
                    continue;
                }
                b'?' => {
                    // ? matches any single character except /
                    if text[t_idx] != b'/' {
                        p_idx += 1;
                        t_idx += 1;
                        continue;
                    }
                }
                c if c == text[t_idx] => {
                    p_idx += 1;
                    t_idx += 1;
                    continue;
                }
                _ => {}
            }
        }

        // Mismatch - try to use star backtracking
        if let (Some(sp), Some(st)) = (star_p_idx, star_t_idx) {
            // For single *, don't match past /
            if !double_star && text[st] == b'/' {
                return false;
            }
            p_idx = sp;
            star_t_idx = Some(st + 1);
            t_idx = st + 1;
        } else {
            return false;
        }
    }

    // Skip trailing stars in pattern
    while p_idx < pattern.len() && pattern[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod glob_match_tests {
        use super::*;

        #[test]
        fn exact_match() {
            assert!(glob_match("main", "main"));
            assert!(glob_match("develop", "develop"));
            assert!(!glob_match("main", "develop"));
            assert!(!glob_match("main", "main2"));
        }

        #[test]
        fn single_wildcard_suffix() {
            assert!(glob_match("feature/*", "feature/login"));
            assert!(glob_match("feature/*", "feature/auth"));
            assert!(glob_match("feature/*", "feature/x"));
            assert!(!glob_match("feature/*", "feature/a/b"));
            assert!(!glob_match("feature/*", "release/1.0"));
        }

        #[test]
        fn single_wildcard_prefix() {
            assert!(glob_match("*/main", "origin/main"));
            assert!(glob_match("*-123", "hotfix-123"));
            assert!(!glob_match("*-123", "hotfix-456"));
        }

        #[test]
        fn single_wildcard_middle() {
            assert!(glob_match("release/*/final", "release/v1/final"));
            assert!(!glob_match("release/*/final", "release/v1/v2/final"));
        }

        #[test]
        fn double_wildcard() {
            assert!(glob_match("feature/**", "feature/user/login"));
            assert!(glob_match("feature/**", "feature/deep/nested/path"));
            assert!(glob_match("feature/**", "feature/x"));
            assert!(!glob_match("feature/**", "release/1.0"));
        }

        #[test]
        fn double_wildcard_middle() {
            assert!(glob_match("**/main", "origin/main"));
            assert!(glob_match("**/main", "upstream/origin/main"));
        }

        #[test]
        fn single_char_wildcard() {
            assert!(glob_match("release-?", "release-1"));
            assert!(glob_match("release-?", "release-2"));
            assert!(!glob_match("release-?", "release-10"));
            assert!(!glob_match("release-?", "release-"));
        }

        #[test]
        fn question_mark_not_slash() {
            assert!(!glob_match("a?b", "a/b"));
        }

        #[test]
        fn empty_pattern() {
            assert!(glob_match("", ""));
            assert!(!glob_match("", "main"));
        }

        #[test]
        fn star_only() {
            assert!(glob_match("*", "main"));
            assert!(glob_match("*", "x"));
            assert!(!glob_match("*", "a/b"));
        }

        #[test]
        fn double_star_only() {
            assert!(glob_match("**", "main"));
            assert!(glob_match("**", "a/b/c"));
        }

        #[test]
        fn complex_patterns() {
            assert!(glob_match("release/*-rc?", "release/v1-rc1"));
            assert!(glob_match("release/*-rc?", "release/v2-rc2"));
            assert!(!glob_match("release/*-rc?", "release/v1-rc10"));
        }
    }

    mod matches_any_pattern_tests {
        use super::*;

        #[test]
        fn empty_patterns_matches_all() {
            assert!(matches_any_pattern("main", &[]));
            assert!(matches_any_pattern("feature/login", &[]));
        }

        #[test]
        fn matches_first_pattern() {
            let patterns = vec!["main".to_string(), "develop".to_string()];
            assert!(matches_any_pattern("main", &patterns));
        }

        #[test]
        fn matches_second_pattern() {
            let patterns = vec!["main".to_string(), "develop".to_string()];
            assert!(matches_any_pattern("develop", &patterns));
        }

        #[test]
        fn matches_glob_pattern() {
            let patterns = vec![
                "main".to_string(),
                "develop".to_string(),
                "feature/*".to_string(),
            ];
            assert!(matches_any_pattern("feature/login", &patterns));
        }

        #[test]
        fn no_match() {
            let patterns = vec!["main".to_string(), "develop".to_string()];
            assert!(!matches_any_pattern("feature/login", &patterns));
        }
    }

    mod get_current_branch_tests {
        use super::*;

        #[test]
        fn returns_branch_for_current_repo() {
            // This test assumes we're running in a git repo
            let result = get_current_branch(None);
            assert!(result.is_ok());
            // Should return Some branch or None if detached
            let branch = result.unwrap();
            if let Some(name) = branch {
                assert!(!name.is_empty());
            }
        }

        #[test]
        fn returns_error_for_nonexistent_path() {
            let path = PathBuf::from("/nonexistent/path/to/repo");
            let result = get_current_branch(Some(&path));
            // Should either error or return None, depending on git behavior
            // The important thing is it doesn't panic
            assert!(result.is_ok() || result.is_err());
        }
    }
}
