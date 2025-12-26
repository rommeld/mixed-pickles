import mixed_pickles
import pytest


class TestAnalyzeCommits:
    """Tests for the analyze_commits function."""

    def test_default_arguments(self):
        """Should succeed with default arguments on current repo."""
        # Should not raise any exception
        mixed_pickles.analyze_commits()

    def test_with_explicit_path(self):
        """Should succeed with explicit path to current repo."""
        mixed_pickles.analyze_commits(path=".")

    def test_with_limit(self):
        """Should succeed with limit argument."""
        mixed_pickles.analyze_commits(limit=5)

    def test_with_threshold(self):
        """Should succeed with custom threshold."""
        mixed_pickles.analyze_commits(threshold=50)

    def test_with_all_arguments(self):
        """Should succeed with all arguments specified."""
        mixed_pickles.analyze_commits(path=".", limit=10, threshold=25)

    def test_limit_zero(self):
        """Should handle limit of zero."""
        mixed_pickles.analyze_commits(limit=0)

    def test_threshold_zero(self):
        """Should handle threshold of zero."""
        mixed_pickles.analyze_commits(threshold=0)

    def test_large_threshold(self):
        """Should handle large threshold value."""
        mixed_pickles.analyze_commits(threshold=1000)


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
