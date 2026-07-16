use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, warn};

pub fn move_file(source: &Path, folder: &str, base_dir: &Path, dry_run: bool) -> Result<()> {
    let filename = source.file_name()
        .context("Failed to get filename")?;

    let dest_dir = base_dir.join(folder);
    let dest_path = dest_dir.join(filename);

    // Safety: check source is within base directory
    if !source.starts_with(base_dir) {
        anyhow::bail!("Source path is outside base directory");
    }

    // Safety: check destination is within base directory
    if !dest_path.starts_with(base_dir) {
        anyhow::bail!("Destination path is outside base directory");
    }

    if dry_run {
        info!("[DRY RUN] Would move {} → {}", source.display(), dest_path.display());
        return Ok(());
    }

    // Create destination directory if it doesn't exist
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)
            .context("Failed to create destination directory")?;
    }

    // Check if destination already exists
    if dest_path.exists() {
        warn!("Destination already exists: {}, skipping", dest_path.display());
        return Ok(());
    }

    // Move the file (with cross-filesystem fallback)
    if let Err(_e) = std::fs::rename(source, &dest_path) {
        std::fs::copy(source, &dest_path)
            .context("Failed to copy file")?;
        std::fs::remove_file(source)
            .context("Failed to remove original file")?;
    }

    info!("Moved {} → {}", source.display(), dest_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_move_file_dry_run() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("test.pdf");
        std::fs::write(&source, "content").unwrap();

        let result = move_file(&source, "Documents", dir.path(), true);
        assert!(result.is_ok());
        assert!(source.exists());
    }

    #[test]
    fn test_move_file_actual() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("test.pdf");
        std::fs::write(&source, "content").unwrap();

        let result = move_file(&source, "Documents", dir.path(), false);
        assert!(result.is_ok());
        assert!(!source.exists());
        assert!(dir.path().join("Documents/test.pdf").exists());
    }

    #[test]
    fn test_move_file_outside_base_dir() {
        let dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();
        let source = outside_dir.path().join("test.pdf");
        std::fs::write(&source, "content").unwrap();

        let result = move_file(&source, "Documents", dir.path(), false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside base directory"));
    }

    #[test]
    fn test_move_file_destination_exists() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("test.pdf");
        std::fs::write(&source, "content").unwrap();

        let dest_dir = dir.path().join("Documents");
        std::fs::create_dir_all(&dest_dir).unwrap();
        std::fs::write(dest_dir.join("test.pdf"), "existing").unwrap();

        let result = move_file(&source, "Documents", dir.path(), false);
        assert!(result.is_ok());
        assert!(source.exists());
    }
}
