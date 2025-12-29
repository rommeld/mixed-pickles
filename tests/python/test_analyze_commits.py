import mixed_pickles
import pytest


class TestAnalyzeCommits:
    """Tests for the analyze_commits function."""

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

    def test_validates_missing_reference(self):
        """Should detect commits missing issue references."""
        # Most real commits lack issue references, so this should raise
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(limit=5, threshold=0)

    def test_validates_invalid_format(self):
        """Should detect commits with invalid format."""
        # Real commits may lack conventional commit format
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(limit=5, threshold=0)

    def test_with_path_validates_commits(self):
        """Should validate commits when using explicit path."""
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(path=".", limit=5, threshold=0)


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
