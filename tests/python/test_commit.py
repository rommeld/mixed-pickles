import mixed_pickles
import pytest


class TestFetchCommits:
    """Tests for the fetch_commits function."""

    def test_fetch_commits_returns_list(self):
        """Should return a list of Commit objects."""
        commits = mixed_pickles.fetch_commits(limit=5)
        assert isinstance(commits, list)
        assert len(commits) <= 5

    def test_fetch_commits_with_path(self):
        """Should work with explicit path."""
        commits = mixed_pickles.fetch_commits(path=".", limit=3)
        assert isinstance(commits, list)

    def test_fetch_commits_limit_zero(self):
        """Should return empty list with limit=0."""
        commits = mixed_pickles.fetch_commits(limit=0)
        assert commits == []

    def test_fetch_commits_non_existent_path(self):
        """Should raise error for non-existent path."""
        with pytest.raises(RuntimeError, match="does not exist|PathNotFound"):
            mixed_pickles.fetch_commits(path="/this/path/does/not/exist")

    def test_fetch_commits_not_a_repository(self):
        """Should raise error when path is not a git repository."""
        with pytest.raises(RuntimeError, match="not a git repository|NotARepository"):
            mixed_pickles.fetch_commits(path="/tmp")


class TestCommit:
    """Tests for the Commit class."""

    @pytest.fixture
    def commit(self):
        """Get a single commit for testing."""
        commits = mixed_pickles.fetch_commits(limit=1)
        assert len(commits) == 1
        return commits[0]

    def test_commit_has_hash(self, commit):
        """Commit should have a 40-character hash."""
        assert len(commit.hash) == 40
        assert all(c in "0123456789abcdef" for c in commit.hash)

    def test_commit_has_author_name(self, commit):
        """Commit should have an author name."""
        assert isinstance(commit.author_name, str)
        assert len(commit.author_name) > 0

    def test_commit_has_author_email(self, commit):
        """Commit should have an author email."""
        assert isinstance(commit.author_email, str)
        assert "@" in commit.author_email

    def test_commit_has_subject(self, commit):
        """Commit should have a subject."""
        assert isinstance(commit.subject, str)


class TestValidation:
    """Tests for the Validation enum and commit validation."""

    def test_validation_enum_exists(self):
        """Validation enum should be importable."""
        assert hasattr(mixed_pickles, "Validation")

    def test_validation_short_commit(self):
        """Should have ShortCommit variant."""
        assert hasattr(mixed_pickles.Validation, "ShortCommit")

    def test_validation_missing_reference(self):
        """Should have MissingReference variant."""
        assert hasattr(mixed_pickles.Validation, "MissingReference")

    def test_validation_invalid_format(self):
        """Should have InvalidFormat variant."""
        assert hasattr(mixed_pickles.Validation, "InvalidFormat")

    def test_validation_vague_language(self):
        """Should have VagueLanguage variant."""
        assert hasattr(mixed_pickles.Validation, "VagueLanguage")

    def test_validation_wip_commit(self):
        """Should have WipCommit variant."""
        assert hasattr(mixed_pickles.Validation, "WipCommit")

    def test_validation_non_imperative(self):
        """Should have NonImperative variant."""
        assert hasattr(mixed_pickles.Validation, "NonImperative")

    def test_validation_str(self):
        """Validation should have human-readable string representation."""
        assert str(mixed_pickles.Validation.ShortCommit) == "Short commit message"
        assert (
            "issue reference" in str(mixed_pickles.Validation.MissingReference).lower()
        )
        assert "format" in str(mixed_pickles.Validation.InvalidFormat).lower()
        assert "vague" in str(mixed_pickles.Validation.VagueLanguage).lower()
        assert "wip" in str(mixed_pickles.Validation.WipCommit).lower()
        assert "imperative" in str(mixed_pickles.Validation.NonImperative).lower()

    def test_validation_repr(self):
        """Validation should have debug representation."""
        assert repr(mixed_pickles.Validation.ShortCommit) == "Validation.ShortCommit"
        assert (
            repr(mixed_pickles.Validation.MissingReference)
            == "Validation.MissingReference"
        )
        assert (
            repr(mixed_pickles.Validation.InvalidFormat) == "Validation.InvalidFormat"
        )
        assert (
            repr(mixed_pickles.Validation.VagueLanguage) == "Validation.VagueLanguage"
        )
        assert repr(mixed_pickles.Validation.WipCommit) == "Validation.WipCommit"
        assert (
            repr(mixed_pickles.Validation.NonImperative) == "Validation.NonImperative"
        )

    def test_validation_equality(self):
        """Validation variants should be comparable."""
        assert (
            mixed_pickles.Validation.ShortCommit == mixed_pickles.Validation.ShortCommit
        )
        assert (
            mixed_pickles.Validation.ShortCommit
            != mixed_pickles.Validation.MissingReference
        )
        assert (
            mixed_pickles.Validation.MissingReference
            != mixed_pickles.Validation.InvalidFormat
        )
        assert (
            mixed_pickles.Validation.InvalidFormat
            != mixed_pickles.Validation.VagueLanguage
        )
        assert (
            mixed_pickles.Validation.VagueLanguage != mixed_pickles.Validation.WipCommit
        )
        assert (
            mixed_pickles.Validation.WipCommit != mixed_pickles.Validation.NonImperative
        )


class TestCommitValidate:
    """Tests for Commit.validate() method."""

    @pytest.fixture
    def commit(self):
        """Get a single commit for testing."""
        commits = mixed_pickles.fetch_commits(limit=1)
        assert len(commits) == 1
        return commits[0]

    def test_validate_returns_list(self, commit):
        """validate() should return a list."""
        failures = commit.validate()
        assert isinstance(failures, list)

    def test_validate_with_high_threshold(self, commit):
        """validate() with high threshold should return ShortCommit."""
        failures = commit.validate(threshold=1000)
        assert mixed_pickles.Validation.ShortCommit in failures

    def test_validate_with_low_threshold(self, commit):
        """validate() with threshold=0 should return empty list."""
        failures = commit.validate(threshold=0)
        assert mixed_pickles.Validation.ShortCommit not in failures

    def test_validate_default_threshold(self, commit):
        """validate() should use default threshold of 30."""
        failures = commit.validate()
        # Just verify it runs without error
        assert isinstance(failures, list)
