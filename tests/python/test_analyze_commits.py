import mixed_pickles
import pytest


class TestAnalyzeCommits:
    """Tests for the analyze_commits function."""

    def test_no_short_commits_with_low_threshold(self):
        """Should succeed when no commits are below threshold."""
        # With threshold=0, no commits should be "short"
        mixed_pickles.analyze_commits(threshold=0)

    def test_no_short_commits_with_low_threshold_and_path(self):
        """Should succeed with explicit path when no short commits."""
        mixed_pickles.analyze_commits(path=".", threshold=0)

    def test_quiet_mode_no_issues(self):
        """Should succeed silently when no issues and quiet=True."""
        mixed_pickles.analyze_commits(threshold=0, quiet=True)

    def test_limit_zero(self):
        """Should handle limit of zero (no commits analyzed)."""
        mixed_pickles.analyze_commits(limit=0)

    def test_raises_when_short_commits_found(self):
        """Should raise RuntimeError when validation issues are found."""
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(threshold=1000)

    def test_raises_with_quiet_when_issues_found(self):
        """Should still raise even in quiet mode when issues found."""
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(threshold=1000, quiet=True)

    def test_with_limit_and_threshold(self):
        """Should work with limit and low threshold."""
        mixed_pickles.analyze_commits(limit=5, threshold=0)


class TestAnalyzeCommitsErrors:
    """Tests for error handling in analyze_commits."""

    def test_non_existent_path(self):
        """Should raise error for non-existent path."""
        with pytest.raises(RuntimeError, match="does not exist|PathNotFound"):
            mixed_pickles.analyze_commits(path="/this/path/does/not/exist")

    def test_path_not_a_repository(self):
        """Should raise error when path is not a git repository."""
        with pytest.raises(RuntimeError, match="not a git repository|NotARepository"):
            mixed_pickles.analyze_commits(path="/tmp")
