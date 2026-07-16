use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::Parser;
use tokio::sync::mpsc;
use tracing::{info, error, warn, debug};

use ifo::{config, watcher, pipeline, executor, FileEvent};

#[derive(Parser)]
#[command(name = "ifo")]
#[command(about = "Intelligent File Organizer")]
struct Cli {
    /// Directory to watch
    #[arg(short, long)]
    dir: PathBuf,

    /// Config file path (default: config.toml)
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let config = config::Config::load(&cli.config)
        .context("Failed to load config")?;

    info!("Config loaded from: {}", cli.config.display());

    let (tx, rx) = mpsc::channel(100);

    let watch_dir = cli.dir.clone();
    std::thread::spawn(move || {
        if let Err(e) = watcher::start_watching(watch_dir, tx) {
            error!("Watcher failed: {}", e);
        }
    });

    let base_dir = cli.dir.canonicalize()
        .context("Failed to resolve base directory")?;
    tokio::runtime::Runtime::new()?.block_on(async {
        process_events(rx, &config, &base_dir, cli.dry_run).await
    })
}

async fn process_events(
    mut rx: mpsc::Receiver<FileEvent>,
    config: &config::Config,
    base_dir: &std::path::Path,
    dry_run: bool,
) -> Result<()> {
    info!("Processing events...");

    while let Some(event) = rx.recv().await {
        debug!("Received event: {:?}", event);

        match pipeline::classify(&event, config) {
            Some(folder) => {
                if let Err(e) = executor::move_file(&event.path, &folder, base_dir, dry_run) {
                    error!("Failed to move {}: {}", event.path.display(), e);
                }
            }
            None => {
                warn!("No rule for extension: {:?}", event.extension);
            }
        }
    }

    Ok(())
}
