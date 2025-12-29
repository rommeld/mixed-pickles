import mixed_pickles
import pytest


class TestValidationConfig:
    """Tests for the ValidationConfig class."""

    def test_config_exists(self):
        """ValidationConfig should be importable."""
        assert hasattr(mixed_pickles, "ValidationConfig")

    def test_config_default_constructor(self):
        """Should create config with default values."""
        config = mixed_pickles.ValidationConfig()
        assert config.threshold == 30
        assert config.require_issue_ref is True
        assert config.require_conventional_format is True
        assert config.check_vague_language is True
        assert config.check_wip is True
        assert config.check_imperative is True

    def test_config_custom_threshold(self):
        """Should accept custom threshold in constructor."""
        config = mixed_pickles.ValidationConfig(threshold=50)
        assert config.threshold == 50

    def test_config_disable_issue_ref(self):
        """Should accept require_issue_ref=False in constructor."""
        config = mixed_pickles.ValidationConfig(require_issue_ref=False)
        assert config.require_issue_ref is False

    def test_config_disable_conventional_format(self):
        """Should accept require_conventional_format=False in constructor."""
        config = mixed_pickles.ValidationConfig(require_conventional_format=False)
        assert config.require_conventional_format is False

    def test_config_disable_vague_language(self):
        """Should accept check_vague_language=False in constructor."""
        config = mixed_pickles.ValidationConfig(check_vague_language=False)
        assert config.check_vague_language is False

    def test_config_disable_wip(self):
        """Should accept check_wip=False in constructor."""
        config = mixed_pickles.ValidationConfig(check_wip=False)
        assert config.check_wip is False

    def test_config_disable_imperative(self):
        """Should accept check_imperative=False in constructor."""
        config = mixed_pickles.ValidationConfig(check_imperative=False)
        assert config.check_imperative is False

    def test_config_multiple_options(self):
        """Should accept multiple options in constructor."""
        config = mixed_pickles.ValidationConfig(
            threshold=50,
            require_issue_ref=False,
            check_wip=False,
        )
        assert config.threshold == 50
        assert config.require_issue_ref is False
        assert config.check_wip is False
        # Others should still be default
        assert config.require_conventional_format is True
        assert config.check_vague_language is True
        assert config.check_imperative is True

    def test_config_mutable_threshold(self):
        """Should allow modifying threshold after creation."""
        config = mixed_pickles.ValidationConfig()
        config.threshold = 100
        assert config.threshold == 100

    def test_config_mutable_require_issue_ref(self):
        """Should allow modifying require_issue_ref after creation."""
        config = mixed_pickles.ValidationConfig()
        config.require_issue_ref = False
        assert config.require_issue_ref is False

    def test_config_repr(self):
        """Should have readable repr."""
        config = mixed_pickles.ValidationConfig()
        repr_str = repr(config)
        assert "ValidationConfig" in repr_str
        assert "threshold=30" in repr_str


class TestValidationConfigSeverity:
    """Tests for ValidationConfig severity methods."""

    def test_get_severity_default_wip(self):
        """WipCommit should default to Error severity."""
        config = mixed_pickles.ValidationConfig()
        severity = config.get_severity(mixed_pickles.Validation.WipCommit)
        assert severity == mixed_pickles.Severity.Error

    def test_get_severity_default_short(self):
        """ShortCommit should default to Warning severity."""
        config = mixed_pickles.ValidationConfig()
        severity = config.get_severity(mixed_pickles.Validation.ShortCommit)
        assert severity == mixed_pickles.Severity.Warning

    def test_get_severity_default_missing_ref(self):
        """MissingReference should default to Info severity."""
        config = mixed_pickles.ValidationConfig()
        severity = config.get_severity(mixed_pickles.Validation.MissingReference)
        assert severity == mixed_pickles.Severity.Info

    def test_set_severity(self):
        """Should allow changing severity of validation."""
        config = mixed_pickles.ValidationConfig()
        config.set_severity(
            mixed_pickles.Validation.MissingReference, mixed_pickles.Severity.Error
        )
        severity = config.get_severity(mixed_pickles.Validation.MissingReference)
        assert severity == mixed_pickles.Severity.Error


class TestSeverityEnum:
    """Tests for the Severity enum."""

    def test_severity_exists(self):
        """Severity enum should be importable."""
        assert hasattr(mixed_pickles, "Severity")

    def test_severity_error(self):
        """Should have Error variant."""
        assert hasattr(mixed_pickles.Severity, "Error")

    def test_severity_warning(self):
        """Should have Warning variant."""
        assert hasattr(mixed_pickles.Severity, "Warning")

    def test_severity_info(self):
        """Should have Info variant."""
        assert hasattr(mixed_pickles.Severity, "Info")

    def test_severity_ignore(self):
        """Should have Ignore variant."""
        assert hasattr(mixed_pickles.Severity, "Ignore")

    def test_severity_str(self):
        """Severity should have string representation."""
        assert str(mixed_pickles.Severity.Error) == "error"
        assert str(mixed_pickles.Severity.Warning) == "warning"
        assert str(mixed_pickles.Severity.Info) == "info"
        assert str(mixed_pickles.Severity.Ignore) == "ignore"

    def test_severity_repr(self):
        """Severity should have repr representation."""
        assert repr(mixed_pickles.Severity.Error) == "Severity.Error"
        assert repr(mixed_pickles.Severity.Warning) == "Severity.Warning"


class TestAnalyzeCommitsWithConfig:
    """Tests for analyze_commits with ValidationConfig."""

    def test_analyze_with_config_no_issue_ref(self):
        """Should not raise when issue ref check is disabled."""
        config = mixed_pickles.ValidationConfig(require_issue_ref=False)
        # This should not raise for missing references
        # We analyze with limit=0 to avoid any actual validation
        mixed_pickles.analyze_commits(limit=0, config=config)

    def test_analyze_with_config_custom_threshold(self):
        """Should respect custom threshold from config."""
        config = mixed_pickles.ValidationConfig(threshold=0)
        # With threshold=0, ShortCommit should never trigger
        config.set_severity(
            mixed_pickles.Validation.ShortCommit, mixed_pickles.Severity.Error
        )
        mixed_pickles.analyze_commits(limit=1, config=config)

    def test_analyze_with_disabled_checks(self):
        """Should skip validation when checks are disabled."""
        config = mixed_pickles.ValidationConfig(
            require_issue_ref=False,
            require_conventional_format=False,
            check_vague_language=False,
            check_wip=False,
            check_imperative=False,
            threshold=0,
        )
        # With all checks disabled and threshold=0, nothing should fail
        mixed_pickles.analyze_commits(limit=5, config=config)

    def test_analyze_with_severity_error_raises(self):
        """Should raise when validation set to Error severity fails."""
        config = mixed_pickles.ValidationConfig(threshold=1000)
        config.set_severity(
            mixed_pickles.Validation.ShortCommit, mixed_pickles.Severity.Error
        )
        with pytest.raises(RuntimeError, match="validation issues"):
            mixed_pickles.analyze_commits(limit=5, config=config)

    def test_analyze_with_severity_ignore_does_not_raise(self):
        """Should not raise when validation set to Ignore severity."""
        config = mixed_pickles.ValidationConfig(threshold=1000)
        config.set_severity(
            mixed_pickles.Validation.ShortCommit, mixed_pickles.Severity.Ignore
        )
        # Even with high threshold, Ignore severity means no error
        mixed_pickles.analyze_commits(limit=1, config=config)
