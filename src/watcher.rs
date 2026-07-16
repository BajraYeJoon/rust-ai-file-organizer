use std::path::PathBuf;
use anyhow::{Result, Context};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use crate::FileEvent;

pub fn start_watching(
    dir: PathBuf,
    tx: mpsc::Sender<FileEvent>,
) -> Result<()> {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    let mut watcher = notify::recommended_watcher(notify_tx)
        .context("Failed to create file watcher")?;

    watcher.watch(&dir, RecursiveMode::Recursive)
        .context("Failed to start watching directory")?;

    info!("Watching directory: {}", dir.display());

    // Spawn thread to process notify events
    std::thread::spawn(move || {
        for res in notify_rx {
            match res {
                Ok(event) => {
                    if let Err(e) = process_notify_event(event, &tx) {
                        warn!("Failed to process event: {}", e);
                    }
                }
                Err(e) => warn!("Watch error: {}", e),
            }
        }
    });

    // Keep watcher alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn process_notify_event(event: Event, tx: &mpsc::Sender<FileEvent>) -> Result<()> {
    if !matches!(event.kind, EventKind::Create(_)) {
        return Ok(());
    }

    for path in event.paths {
        if path.is_file() {
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e));

            debug!("File created: {}", path.display());

            let file_event = FileEvent {
                path: path.clone(),
                extension,
            };

            tx.blocking_send(file_event)
                .context("Failed to send file event")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_notify_event_skips_directories() {
        let (tx, mut rx) = mpsc::channel(10);
        let dir = tempfile::tempdir().unwrap();

        let event = Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![dir.path().to_path_buf()],
            attrs: Default::default(),
        };

        process_notify_event(event, &tx).unwrap();
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_process_notify_event_sends_file_event() {
        let (tx, mut rx) = mpsc::channel(10);
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.pdf");
        std::fs::write(&file_path, "content").unwrap();

        let event = Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![file_path.clone()],
            attrs: Default::default(),
        };

        process_notify_event(event, &tx).unwrap();
        let file_event = rx.try_recv().unwrap();
        assert_eq!(file_event.path, file_path);
        assert_eq!(file_event.extension, Some(".pdf".to_string()));
    }
}
