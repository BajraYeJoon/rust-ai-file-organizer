use crate::config::Config;
use crate::FileEvent;

pub fn classify(event: &FileEvent, config: &Config) -> Option<String> {
    let extension = event.extension.as_deref()?;
    config.get_folder(extension).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_config() -> Config {
        let mut rules = HashMap::new();
        rules.insert(".pdf".to_string(), "Documents/PDFs".to_string());
        rules.insert(".jpg".to_string(), "Images".to_string());
        
        Config {
            rules,
        }
    }

    #[test]
    fn test_classify_pdf() {
        let config = make_config();
        let event = FileEvent {
            path: "/tmp/report.pdf".into(),
            extension: Some(".pdf".to_string()),
        };
        assert_eq!(classify(&event, &config), Some("Documents/PDFs".to_string()));
    }

    #[test]
    fn test_classify_unknown_extension() {
        let config = make_config();
        let event = FileEvent {
            path: "/tmp/file.xyz".into(),
            extension: Some(".xyz".to_string()),
        };
        assert_eq!(classify(&event, &config), None);
    }

    #[test]
    fn test_classify_no_extension() {
        let config = make_config();
        let event = FileEvent {
            path: "/tmp/Makefile".into(),
            extension: None,
        };
        assert_eq!(classify(&event, &config), None);
    }
}
