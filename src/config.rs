use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::Deserialize;

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        // If config exists, load it
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .context("Failed to read config file")?;
            
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            
            return Ok(config);
        }

        // First run: create default config at ~/.config/ifo/config.toml
        let default_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ifo")
            .join("config.toml");

        if default_path.exists() {
            let content = std::fs::read_to_string(&default_path)
                .context("Failed to read default config")?;
            
            let config: Config = toml::from_str(&content)
                .context("Failed to parse default config")?;
            
            return Ok(config);
        }

        // Create default config
        if let Some(parent) = default_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        std::fs::write(&default_path, DEFAULT_CONFIG)
            .context("Failed to write default config")?;

        println!("Created default config at: {}", default_path.display());
        println!("Edit it to customize your rules.");

        let config: Config = toml::from_str(DEFAULT_CONFIG)
            .context("Failed to parse default config")?;
        
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

        let config = Config::load(&temp_file.path().to_path_buf()).unwrap();
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
