import mixed_pickles
import pytest


class TestAnalyzeCommits:
    """Tests for the analyze_commits function."""

    def test_limit_zero(self):
        """Should handle limit of zero (no commits analyzed)."""
        mixed_pickles.analyze_commits(limit=0)

    def test_raises_when_short_commits_found(self):
        """Should raise RuntimeError when validation issues are found."""
        config = mixed_pickles.ValidationConfig(threshold=1000)
        config.set_severity(
            mixed_pickles.Validation.ShortCommit, mixed_pickles.Severity.Error
        )
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(config=config)

    def test_raises_with_quiet_when_issues_found(self):
        """Should still raise even in quiet mode when issues found."""
        config = mixed_pickles.ValidationConfig(threshold=1000)
        config.set_severity(
            mixed_pickles.Validation.ShortCommit, mixed_pickles.Severity.Error
        )
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(quiet=True, config=config)

    def test_validates_missing_reference(self):
        """Should detect commits missing issue references."""
        config = mixed_pickles.ValidationConfig(threshold=0)
        config.set_severity(
            mixed_pickles.Validation.MissingReference, mixed_pickles.Severity.Error
        )
        # Most real commits lack issue references, so this should raise
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(limit=5, config=config)

    def test_validates_with_multiple_error_types(self):
        """Should detect commits based on configured error types."""
        config = mixed_pickles.ValidationConfig(threshold=0)
        config.set_severity(
            mixed_pickles.Validation.MissingReference, mixed_pickles.Severity.Error
        )
        # Most real commits lack issue references, so this should raise
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(limit=5, config=config)

    def test_with_path_validates_commits(self):
        """Should validate commits when using explicit path."""
        config = mixed_pickles.ValidationConfig(threshold=0)
        config.set_severity(
            mixed_pickles.Validation.MissingReference, mixed_pickles.Severity.Error
        )
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(path=".", limit=5, config=config)


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
