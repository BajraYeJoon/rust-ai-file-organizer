use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    pub fn get_folder(&self, extension: &str) -> Option<&String> {
        self.rules.get(extension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_valid_config() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"
[rules]
".pdf" = "Documents"
"#).unwrap();

        let config = Config::load(temp_file.path()).unwrap();
        assert_eq!(config.get_folder(".pdf"), Some(&"Documents".to_string()));
    }

    #[test]
    fn test_get_folder_unknown_extension() {
        let config = Config {
            rules: HashMap::new(),
        };
        assert_eq!(config.get_folder(".unknown"), None);
    }
}
