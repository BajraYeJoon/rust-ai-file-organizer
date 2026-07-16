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

fn get_install_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        Ok(dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".local")
            .join("bin"))
    }

    #[cfg(target_os = "windows")]
    {
        Ok(dirs::data_local_dir()
            .context("Failed to get local data directory")?
            .join("ifo")
            .join("bin"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".local")
            .join("bin"))
    }
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

    let current_binary = get_binary_path()?;
    let config_dir = get_config_dir();
    let install_dir = get_install_dir()?;

    // Create directories
    std::fs::create_dir_all(&install_dir)
        .context("Failed to create install directory")?;
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    // Install binary
    let installed_binary = if cfg!(target_os = "windows") {
        install_dir.join("ifo.exe")
    } else {
        install_dir.join("ifo")
    };

    std::fs::copy(&current_binary, &installed_binary)
        .context("Failed to copy binary")?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&installed_binary, perms)?;
    }

    println!("Installed binary: {}", installed_binary.display());

    // Create default config if not exists
    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        std::fs::write(&config_file, include_str!("../config.toml"))
            .context("Failed to write default config")?;
        println!("Created config: {}", config_file.display());
    }

    // Platform-specific service setup
    #[cfg(target_os = "linux")]
    {
        setup_systemd(&installed_binary, &watch_dir, &config_dir)?;
    }

    #[cfg(target_os = "windows")]
    {
        setup_windows_service(&installed_binary, &watch_dir)?;
    }

    println!();
    println!("Installed!");
    println!();
    println!("Watching: {}", watch_dir.display());
    println!("Config:   {}", config_file.display());
    println!();

    // Platform-specific PATH instructions
    #[cfg(target_os = "linux")]
    {
        let path_entry = format!("export PATH=\"{}:$PATH\"", install_dir.display());
        let bashrc = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".bashrc");

        if bashrc.exists() {
            let content = std::fs::read_to_string(&bashrc).unwrap_or_default();
            if !content.contains(&path_entry) {
                println!("Add to PATH (run once):");
                println!("  echo '{}' >> ~/.bashrc && source ~/.bashrc", path_entry);
                println!();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        println!("Add to PATH (run in PowerShell as Admin):");
        println!("  [Environment]::SetEnvironmentVariable('Path', '{};' + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')", install_dir.display());
        println!();
        println!("NOTE: Restart your terminal after adding to PATH.");
    }

    println!("Commands:");
    println!("  ifo status    - Check if running");
    println!("  ifo stop      - Stop the service");
    println!("  ifo start     - Start the service");
    println!("  ifo uninstall - Remove everything");

    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_systemd(binary: &PathBuf, watch_dir: &PathBuf, config_dir: &PathBuf) -> Result<()> {
    let service_dir = config_dir
        .parent()
        .unwrap_or(config_dir)
        .parent()
        .unwrap_or(config_dir)
        .join("systemd")
        .join("user");

    std::fs::create_dir_all(&service_dir)
        .context("Failed to create systemd directory")?;

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

    // Enable linger for auto-start on login
    let _ = std::process::Command::new("loginctl")
        .args(["enable-linger"])
        .status();

    println!("Service installed and started.");

    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_windows_service(binary: &PathBuf, watch_dir: &PathBuf) -> Result<()> {
    // Add to Windows startup via registry
    let reg_cmd = format!(
        "reg add HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run /v ifo /t REG_SZ /d \"\\\"{}\\\" --dir \\\"{}\\\"\" /f",
        binary.display(),
        watch_dir.display()
    );

    std::process::Command::new("cmd")
        .args(["/C", &reg_cmd])
        .status()
        .context("Failed to add to startup")?;

    println!("Added to Windows startup.");

    // Start it now
    std::process::Command::new(binary)
        .args(["--dir", watch_dir.to_str().unwrap_or("")])
        .spawn()
        .context("Failed to start ifo")?;

    println!("IFO started.");

    Ok(())
}

fn uninstall() -> Result<()> {
    println!("IFO - Uninstalling");
    println!("==================");
    println!();

    // Stop service (Linux)
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", "ifo"])
            .status();
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", "ifo"])
            .status();

        let service_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("systemd")
            .join("user")
            .join("ifo.service");

        if service_file.exists() {
            std::fs::remove_file(&service_file)?;
            println!("Removed service");
        }

        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
    }

    // Remove from Windows startup
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "reg delete HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run /v ifo /f"])
            .status();

        // Kill running process
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", "ifo.exe", "/F"])
            .status();

        println!("Removed from startup");
    }

    // Remove binary
    let install_dir = get_install_dir()?;
    let binary_name = if cfg!(target_os = "windows") {
        "ifo.exe"
    } else {
        "ifo"
    };
    let installed_binary = install_dir.join(binary_name);

    if installed_binary.exists() {
        std::fs::remove_file(&installed_binary)?;
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
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("systemctl")
            .args(["--user", "status", "ifo"])
            .status()
            .context("Failed to check status")?;

        if output.success() {
            println!("IFO is running");
        } else {
            println!("IFO is not running");
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ifo.exe"])
            .output()
            .context("Failed to check status")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("ifo.exe") {
            println!("IFO is running");
        } else {
            println!("IFO is not running");
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        println!("Status not supported on this platform.");
        println!("Check if ifo process is running.");
    }

    Ok(())
}

fn start() -> Result<()> {
    println!("Starting IFO...");

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("systemctl")
            .args(["--user", "start", "ifo"])
            .status()
            .context("Failed to start service")?;
    }

    #[cfg(target_os = "windows")]
    {
        let install_dir = get_install_dir()?;
        let binary = install_dir.join("ifo.exe");
        std::process::Command::new(binary)
            .spawn()
            .context("Failed to start ifo")?;
    }

    println!("IFO started.");

    Ok(())
}

fn stop() -> Result<()> {
    println!("Stopping IFO...");

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("systemctl")
            .args(["--user", "stop", "ifo"])
            .status()
            .context("Failed to stop service")?;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/IM", "ifo.exe", "/F"])
            .status();
    }

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
