# IFO - Intelligent File Organizer

A background app that watches your Downloads folder and automatically sorts files into organized folders.

## Features

- **Runs in background** - No terminal needed, auto-starts on boot
- **Real-time monitoring** - Watches directories for new files
- **Extension-based sorting** - Moves files to folders based on file type
- **Auto-config** - Creates config on first run, edit to customize
- **Safe operations** - Path validation, atomic moves, cross-filesystem support

## Installation

### Linux (One Command)

```bash
# Download
wget https://github.com/YOUR_USERNAME/ifo/releases/download/v0.1.0/ifo-linux-amd64

# Install (auto-starts on boot)
chmod +x ifo-linux-amd64
./ifo-linux-amd64 install
```

**That's it.** It's now running in background.

### Windows

Download `ifo-windows-amd64.exe` from Releases and run it.

## Usage

```bash
# Install as background service
ifo install

# Check if running
ifo status

# Stop/Start
ifo stop
ifo start

# Uninstall
ifo uninstall

# Run in terminal (foreground)
ifo --dir ~/Downloads

# Test first (no files moved)
ifo --dir ~/Downloads --dry-run
```

## Configuration

Config auto-creates at `~/.config/ifo/config.toml`. Edit it:

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

## How It Works

1. **Watch** - Monitors ~/Downloads for new files
2. **Classify** - Checks file extension against config rules
3. **Move** - Automatically moves file to matching folder
4. **Log** - Records all operations

## Requirements

- Linux x86_64 (Arch, CachyOS, Fedora, Ubuntu, etc.)
- No dependencies needed

## License

MIT License
