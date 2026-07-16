# IFO - Intelligent File Organizer

A fast, CLI-based file organizer that watches directories and automatically sorts files into organized folders based on extension rules.

## Features

- **Real-time monitoring** - Watches directories for new files
- **Extension-based sorting** - Moves files to folders based on file type
- **Dry-run mode** - Preview what would happen without moving files
- **Configurable rules** - Customizable extension-to-folder mapping
- **Safe operations** - Path validation, atomic moves, cross-filesystem support
- **Lightweight** - Single 2.6MB binary, no dependencies needed

## Installation

### Docker (Recommended)

```bash
# Clone the repo
git clone https://github.com/YOUR_USERNAME/ifo.git
cd ifo

# Build and run with docker-compose
docker-compose up -d

# Or run directly
docker build -t ifo .
docker run -d \
  --name ifo \
  -v ~/Downloads:/watch \
  -v $(pwd)/config.toml:/etc/ifo/config.toml:ro \
  ifo --dir /watch
```

### Standalone Binary (No Docker Required)

**Linux (Arch, CachyOS, Fedora, etc.):**
```bash
# Download the static binary (works on any distro)
wget https://github.com/YOUR_USERNAME/ifo/releases/download/v0.1.0/ifo-linux-amd64

# Make it executable
chmod +x ifo-linux-amd64

# Move to PATH (optional)
sudo mv ifo-linux-amd64 /usr/local/bin/ifo
```

**Linux (Ubuntu, Debian - glibc):**
```bash
# Download the glibc binary
wget https://github.com/YOUR_USERNAME/ifo/releases/download/v0.1.0/ifo-linux-amd64-glibc

# Make it executable
chmod +x ifo-linux-amd64-glibc

# Move to PATH (optional)
sudo mv ifo-linux-amd64-glibc /usr/local/bin/ifo
```

**Windows:**
```powershell
# Download ifo-windows-amd64.exe from GitHub Releases
# Move to a folder in your PATH, or run directly
```

### From Source

```bash
# Requires Rust installed (https://rustup.rs)
git clone https://github.com/YOUR_USERNAME/ifo.git
cd ifo
cargo build --release
./target/release/ifo --help
```

## Quick Start

```bash
# 1. Create config file
cat > config.toml << 'EOF'
[rules]
".pdf" = "Documents/PDFs"
".jpg" = "Images"
".png" = "Images"
".mp4" = "Videos"
".zip" = "Archives"
".rs" = "Code/Rust"
".py" = "Code/Python"
EOF

# 2. Test with dry-run (safe, no files moved)
ifo --dir ~/Downloads --dry-run

# 3. Run for real
ifo --dir ~/Downloads
```

## Usage

```
ifo [OPTIONS]

Options:
  -d, --dir <DIR>          Directory to watch
  -c, --config <CONFIG>    Config file path [default: config.toml]
  -v, --verbose            Enable verbose logging
      --dry-run            Dry run mode (show what would happen)
  -h, --help               Print help
```

## Configuration

Edit `config.toml` to define your rules:

```toml
[rules]
# Extension → folder mapping
".pdf" = "Documents/PDFs"
".jpg" = "Images"
".png" = "Images"
".mp4" = "Videos"
".zip" = "Archives"
".rs" = "Code/Rust"
".py" = "Code/Python"
".txt" = "Documents/Text"
".doc" = "Documents/Word"
".docx" = "Documents/Word"
".xlsx" = "Documents/Excel"
".pptx" = "Documents/PowerPoint"
```

## Examples

```bash
# Watch Downloads folder with default config
ifo --dir ~/Downloads

# Watch with custom config
ifo --dir ~/Downloads --config /path/to/config.toml

# Preview mode (no files moved)
ifo --dir ~/Downloads --dry-run

# Verbose logging
ifo --dir ~/Downloads --verbose
```

## How It Works

1. **Watch** - Monitors directory for new file creation events
2. **Classify** - Checks file extension against config rules
3. **Move** - Atomically moves file to matching folder
4. **Log** - Records all operations for transparency

## Safety Features

- Path traversal protection (prevents moving files outside watch directory)
- Atomic file moves (no data corruption)
- Cross-filesystem support (copy+delete fallback)
- Dry-run mode for testing
- Destination conflict handling (skips existing files)

## Requirements

### Docker
- Docker installed (https://docs.docker.com/get-docker/)

### Standalone Binary
- **Linux:** x86_64 (static binary, works on Arch/CachyOS/Fedora/etc.)
- **Linux (glibc):** x86_64 (for Ubuntu/Debian-based distros)
- **Windows:** x86_64 (no dependencies)

## Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/YOUR_USERNAME/ifo.git
cd ifo
cargo build --release

# Binary at: target/release/ifo
```

## License

MIT License

## Contributing

Contributions welcome! Please open an issue or PR.
