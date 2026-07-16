pub mod config;
pub mod pipeline;
pub mod executor;
pub mod watcher;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: PathBuf,
    pub extension: Option<String>,
}
