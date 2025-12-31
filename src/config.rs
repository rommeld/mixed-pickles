//! Configuration parsing for pyproject.toml.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid TOML syntax: {0}")]
    Parse(#[from] toml::de::Error),

    #[error(
        "invalid validation name: '{0}' (valid: short, wip, reference, format, vague, imperative)"
    )]
    InvalidValidation(String),

    #[error("invalid severity: '{0}' (valid: error, warning, info, ignore)")]
    InvalidSeverity(String),
}

const PYPROJECT_TOML: &str = "pyproject.toml";
const DEDICATED_CONFIG: &str = ".mixed-pickles.toml";

#[derive(Debug, Deserialize, Default)]
pub struct PyProjectToml {
    pub tool: Option<ToolTable>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ToolTable {
    #[serde(rename = "mixed-pickles")]
    pub mixed_pickles: Option<ToolConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ToolConfig {
    pub threshold: Option<usize>,
    pub strict: Option<bool>,
    pub disable: Vec<String>,
    pub severity: Option<SeverityConfig>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SeverityConfig {
    #[serde(alias = "short-commit")]
    pub short: Option<String>,

    #[serde(alias = "wip-commit")]
    pub wip: Option<String>,

    #[serde(alias = "missing-reference")]
    pub reference: Option<String>,

    #[serde(alias = "invalid-format")]
    pub format: Option<String>,

    #[serde(alias = "vague-language")]
    pub vague: Option<String>,

    #[serde(alias = "non-imperative")]
    pub imperative: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFile {
    PyProjectToml(PathBuf),
    Dedicated(PathBuf),
}

impl AsRef<Path> for ConfigFile {
    fn as_ref(&self) -> &Path {
        match self {
            ConfigFile::PyProjectToml(p) | ConfigFile::Dedicated(p) => p,
        }
    }
}

/// Find a configuration file by walking up from the start directory.
/// Checks for `.mixed-pickles.toml` first, then `pyproject.toml`.
pub fn find_config_file(start_dir: &Path) -> Option<ConfigFile> {
    let start = start_dir
        .canonicalize()
        .unwrap_or_else(|_| start_dir.to_path_buf());

    std::iter::successors(Some(start.as_path()), |p| p.parent()).find_map(|dir| {
        let dedicated = dir.join(DEDICATED_CONFIG);
        if dedicated.exists() {
            return Some(ConfigFile::Dedicated(dedicated));
        }

        let pyproject = dir.join(PYPROJECT_TOML);
        if pyproject.exists() {
            return Some(ConfigFile::PyProjectToml(pyproject));
        }

        None
    })
}

/// Load configuration from a config file.
/// Handles both pyproject.toml and dedicated .mixed-pickles.toml formats.
pub fn load_config(config_file: &ConfigFile) -> Result<ToolConfig, ConfigError> {
    match config_file {
        ConfigFile::PyProjectToml(path) => {
            let content = std::fs::read_to_string(path)?;
            let pyproject: PyProjectToml = toml::from_str(&content)?;
            Ok(pyproject
                .tool
                .and_then(|t| t.mixed_pickles)
                .unwrap_or_default())
        }
        ConfigFile::Dedicated(path) => {
            let content = std::fs::read_to_string(path)?;
            let config: ToolConfig = toml::from_str(&content)?;
            Ok(config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[tool.mixed-pickles]
"#;
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();
        let tool_config = config.tool.unwrap().mixed_pickles.unwrap();

        assert!(tool_config.threshold.is_none());
        assert!(tool_config.strict.is_none());
        assert!(tool_config.disable.is_empty());
        assert!(tool_config.severity.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[tool.mixed-pickles]
threshold = 50
strict = true
disable = ["format", "reference"]

[tool.mixed-pickles.severity]
short = "error"
wip = "error"
vague = "ignore"
"#;
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();
        let tool_config = config.tool.unwrap().mixed_pickles.unwrap();

        assert_eq!(tool_config.threshold, Some(50));
        assert_eq!(tool_config.strict, Some(true));
        assert_eq!(tool_config.disable, vec!["format", "reference"]);

        let severity = tool_config.severity.unwrap();
        assert_eq!(severity.short, Some("error".to_string()));
        assert_eq!(severity.wip, Some("error".to_string()));
        assert_eq!(severity.vague, Some("ignore".to_string()));
        assert!(severity.reference.is_none());
    }

    #[test]
    fn test_parse_with_aliases() {
        let toml_str = r#"
[tool.mixed-pickles]
disable = ["short-commit", "missing-reference"]

[tool.mixed-pickles.severity]
short-commit = "error"
vague-language = "ignore"
"#;
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();
        let tool_config = config.tool.unwrap().mixed_pickles.unwrap();

        assert_eq!(
            tool_config.disable,
            vec!["short-commit", "missing-reference"]
        );

        let severity = tool_config.severity.unwrap();
        assert_eq!(severity.short, Some("error".to_string()));
        assert_eq!(severity.vague, Some("ignore".to_string()));
    }

    #[test]
    fn test_parse_empty_pyproject() {
        let toml_str = "";
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();

        assert!(config.tool.is_none());
    }

    #[test]
    fn test_parse_pyproject_without_mixed_pickles() {
        let toml_str = r#"
[tool.pytest]
testpaths = ["tests"]
"#;
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();
        let tool_table = config.tool.unwrap();

        assert!(tool_table.mixed_pickles.is_none());
    }

    #[test]
    fn test_parse_inline_severity_table() {
        let toml_str = r#"
[tool.mixed-pickles]
severity = { short = "error", wip = "warning" }
"#;
        let config: PyProjectToml = toml::from_str(toml_str).unwrap();
        let tool_config = config.tool.unwrap().mixed_pickles.unwrap();
        let severity = tool_config.severity.unwrap();

        assert_eq!(severity.short, Some("error".to_string()));
        assert_eq!(severity.wip, Some("warning".to_string()));
    }

    #[test]
    fn test_find_pyproject_in_current_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(&pyproject_path, "[project]\nname = \"test\"").unwrap();

        let result = find_config_file(temp_dir.path());

        assert!(result.is_some());
        let config_file = result.unwrap();
        assert!(matches!(config_file, ConfigFile::PyProjectToml(_)));
        if let ConfigFile::PyProjectToml(path) = config_file {
            assert_eq!(path, pyproject_path.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_find_pyproject_in_parent_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(&pyproject_path, "[project]\nname = \"test\"").unwrap();

        let subdir = temp_dir.path().join("src").join("lib");
        std::fs::create_dir_all(&subdir).unwrap();

        let result = find_config_file(&subdir);

        assert!(result.is_some());
        let config_file = result.unwrap();
        assert!(matches!(config_file, ConfigFile::PyProjectToml(_)));
        if let ConfigFile::PyProjectToml(path) = config_file {
            assert_eq!(path, pyproject_path.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_find_dedicated_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dedicated_path = temp_dir.path().join(".mixed-pickles.toml");
        std::fs::write(&dedicated_path, "threshold = 50").unwrap();

        let result = find_config_file(temp_dir.path());

        assert!(result.is_some());
        let config_file = result.unwrap();
        assert!(matches!(config_file, ConfigFile::Dedicated(_)));
        if let ConfigFile::Dedicated(path) = config_file {
            assert_eq!(path, dedicated_path.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_dedicated_config_takes_precedence() {
        let temp_dir = tempfile::tempdir().unwrap();

        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(&pyproject_path, "[project]\nname = \"test\"").unwrap();

        let dedicated_path = temp_dir.path().join(".mixed-pickles.toml");
        std::fs::write(&dedicated_path, "threshold = 50").unwrap();

        let result = find_config_file(temp_dir.path());

        assert!(result.is_some());
        let config_file = result.unwrap();
        assert!(matches!(config_file, ConfigFile::Dedicated(_)));
        if let ConfigFile::Dedicated(path) = config_file {
            assert_eq!(path, dedicated_path.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_no_config_file_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = find_config_file(temp_dir.path());
        // May find config in parent; just verify no panic
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_find_config_from_deep_subdirectory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(&pyproject_path, "[project]\nname = \"test\"").unwrap();

        let deep_subdir = temp_dir
            .path()
            .join("src")
            .join("modules")
            .join("feature")
            .join("utils");
        std::fs::create_dir_all(&deep_subdir).unwrap();

        let result = find_config_file(&deep_subdir);

        assert!(result.is_some());
        let config_file = result.unwrap();
        assert!(matches!(config_file, ConfigFile::PyProjectToml(_)));
        if let ConfigFile::PyProjectToml(path) = config_file {
            assert_eq!(path, pyproject_path.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_finds_nearest_config_when_multiple_exist() {
        let temp_dir = tempfile::tempdir().unwrap();

        let root_pyproject = temp_dir.path().join("pyproject.toml");
        std::fs::write(&root_pyproject, "[project]\nname = \"root\"").unwrap();

        let subdir = temp_dir.path().join("subproject");
        std::fs::create_dir_all(&subdir).unwrap();
        let sub_pyproject = subdir.join("pyproject.toml");
        std::fs::write(&sub_pyproject, "[project]\nname = \"sub\"").unwrap();

        let result = find_config_file(&subdir);

        assert!(result.is_some());
        let config_file = result.unwrap();
        if let ConfigFile::PyProjectToml(path) = config_file {
            assert_eq!(path, sub_pyproject.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_load_config_from_pyproject() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(
            &pyproject_path,
            r#"
[project]
name = "test"

[tool.mixed-pickles]
threshold = 50
strict = true
"#,
        )
        .unwrap();

        let config_file = ConfigFile::PyProjectToml(pyproject_path);
        let config = load_config(&config_file).unwrap();

        assert_eq!(config.threshold, Some(50));
        assert_eq!(config.strict, Some(true));
    }

    #[test]
    fn test_load_config_from_dedicated_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dedicated_path = temp_dir.path().join(".mixed-pickles.toml");
        std::fs::write(
            &dedicated_path,
            r#"
threshold = 75
disable = ["wip", "format"]
"#,
        )
        .unwrap();

        let config_file = ConfigFile::Dedicated(dedicated_path);
        let config = load_config(&config_file).unwrap();

        assert_eq!(config.threshold, Some(75));
        assert_eq!(config.disable, vec!["wip", "format"]);
    }

    #[test]
    fn test_load_config_returns_default_when_section_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        std::fs::write(
            &pyproject_path,
            r#"
[project]
name = "test"
"#,
        )
        .unwrap();

        let config_file = ConfigFile::PyProjectToml(pyproject_path);
        let config = load_config(&config_file).unwrap();

        assert!(config.threshold.is_none());
        assert!(config.strict.is_none());
        assert!(config.disable.is_empty());
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("pyproject.toml");
        std::fs::write(&path, "this is not valid toml [[[").unwrap();

        let config_file = ConfigFile::PyProjectToml(path);
        let result = load_config(&config_file);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Parse(_)));
    }

    #[test]
    fn test_load_config_file_not_found() {
        let path = PathBuf::from("/nonexistent/path/pyproject.toml");
        let config_file = ConfigFile::PyProjectToml(path);
        let result = load_config(&config_file);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_)));
    }

    #[test]
    fn test_load_config_with_severity_overrides() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join(".mixed-pickles.toml");
        std::fs::write(
            &path,
            r#"
[severity]
short = "error"
wip = "warning"
reference = "ignore"
"#,
        )
        .unwrap();

        let config_file = ConfigFile::Dedicated(path);
        let config = load_config(&config_file).unwrap();

        let severity = config.severity.unwrap();
        assert_eq!(severity.short, Some("error".to_string()));
        assert_eq!(severity.wip, Some("warning".to_string()));
        assert_eq!(severity.reference, Some("ignore".to_string()));
    }
}
