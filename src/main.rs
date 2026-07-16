use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use tokio::sync::mpsc;
use tracing::{info, error, warn, debug};

use ifo::{config, watcher, pipeline, executor, FileEvent};

#[derive(Parser)]
#[command(name = "ifo")]
#[command(about = "Intelligent File Organizer")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directory to watch
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install as background service (auto-starts on boot)
    Install {
        /// Directory to watch [default: ~/Downloads]
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Uninstall background service
    Uninstall,
    /// Show service status
    Status,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Install { dir }) => return install(dir.as_ref()),
        Some(Commands::Uninstall) => return uninstall(),
        Some(Commands::Status) => return status(),
        Some(Commands::Start) => return start(),
        Some(Commands::Stop) => return stop(),
        None => {}
    }

    // Default: run in foreground
    let dir = cli.dir.unwrap_or_else(|| {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
    });

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let config_path = cli.config.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ifo")
            .join("config.toml")
    });

    let config = config::Config::load(&config_path)
        .context("Failed to load config")?;

    info!("Watching: {}", dir.display());
    info!("Config: {}", config_path.display());

    let (tx, rx) = mpsc::channel(100);

    let watch_dir = dir.clone();
    std::thread::spawn(move || {
        if let Err(e) = watcher::start_watching(watch_dir, tx) {
            error!("Watcher failed: {}", e);
        }
    });

    let base_dir = dir.canonicalize()
        .context("Failed to resolve base directory")?;
    tokio::runtime::Runtime::new()?.block_on(async {
        process_events(rx, &config, &base_dir, cli.dry_run).await
    })
}

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ifo")
}

fn get_binary_path() -> Result<PathBuf> {
    let current = std::env::current_exe()
        .context("Failed to get current executable path")?;
    Ok(current)
}

fn install(dir: Option<&PathBuf>) -> Result<()> {
    let watch_dir = dir
        .cloned()
        .unwrap_or_else(|| dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")));

    println!("IFO - Installing background service");
    println!("===================================");
    println!();

    // Get paths
    let binary = get_binary_path()?;
    let config_dir = get_config_dir();
    let service_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("systemd")
        .join("user");

    // Create directories
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    std::fs::create_dir_all(&service_dir)
        .context("Failed to create systemd directory")?;

    // Create default config if not exists
    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        std::fs::write(&config_file, include_str!("../config.toml"))
            .context("Failed to write default config")?;
        println!("Created config: {}", config_file.display());
    }

    // Create service file
    let service_content = format!(
        r#"[Unit]
Description=Intelligent File Organizer
After=network.target

[Service]
Type=simple
ExecStart={} --dir {}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#,
        binary.display(),
        watch_dir.display()
    );

    let service_file = service_dir.join("ifo.service");
    std::fs::write(&service_file, &service_content)
        .context("Failed to write service file")?;

    // Reload and enable
    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .context("Failed to reload systemd")?;

    std::process::Command::new("systemctl")
        .args(["--user", "enable", "ifo"])
        .status()
        .context("Failed to enable service")?;

    std::process::Command::new("systemctl")
        .args(["--user", "start", "ifo"])
        .status()
        .context("Failed to start service")?;

    println!();
    println!("Installed!");
    println!();
    println!("Watching: {}", watch_dir.display());
    println!("Config:   {}", config_file.display());
    println!("Service:  {}", service_file.display());
    println!();
    println!("Commands:");
    println!("  ifo status    - Check if running");
    println!("  ifo stop      - Stop the service");
    println!("  ifo start     - Start the service");
    println!("  ifo uninstall - Remove everything");

    // Enable linger for auto-start on login
    let _ = std::process::Command::new("loginctl")
        .args(["enable-linger"])
        .status();

    Ok(())
}

fn uninstall() -> Result<()> {
    println!("IFO - Uninstalling");
    println!("==================");
    println!();

    // Stop and disable service
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "stop", "ifo"])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "ifo"])
        .status();

    // Remove service file
    let service_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("systemd")
        .join("user")
        .join("ifo.service");

    if service_dir.exists() {
        std::fs::remove_file(&service_dir)?;
        println!("Removed service");
    }

    // Reload systemd
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    // Remove binary
    let binary = get_binary_path()?;
    if binary.exists() {
        std::fs::remove_file(&binary)?;
        println!("Removed binary");
    }

    // Ask about config
    let config_dir = get_config_dir();
    if config_dir.exists() {
        print!("Remove config at {}? [y/N] ", config_dir.display());
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            std::fs::remove_dir_all(&config_dir)?;
            println!("Removed config");
        } else {
            println!("Config kept");
        }
    }

    println!();
    println!("IFO uninstalled.");

    Ok(())
}

fn status() -> Result<()> {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "status", "ifo"])
        .status()
        .context("Failed to check status")?;

    if output.success() {
        println!("IFO is running");
    } else {
        println!("IFO is not running");
    }

    Ok(())
}

fn start() -> Result<()> {
    println!("Starting IFO...");

    std::process::Command::new("systemctl")
        .args(["--user", "start", "ifo"])
        .status()
        .context("Failed to start service")?;

    println!("IFO started.");

    Ok(())
}

fn stop() -> Result<()> {
    println!("Stopping IFO...");

    std::process::Command::new("systemctl")
        .args(["--user", "stop", "ifo"])
        .status()
        .context("Failed to stop service")?;

    println!("IFO stopped.");

    Ok(())
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
