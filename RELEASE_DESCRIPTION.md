## IFO v0.1.0 - Initial Release

A fast CLI file organizer that watches directories and automatically sorts files into organized folders based on extension rules.

### What's New
- Real-time directory monitoring
- Extension-based file sorting
- Dry-run mode for safe testing
- Configurable rules via TOML
- Cross-filesystem support
- Docker support for easy deployment

### Installation

**Docker (Recommended):**
```bash
git clone https://github.com/YOUR_USERNAME/ifo.git
cd ifo
docker-compose up -d
```

**Standalone Binary (No Docker Required):**
- Download the binary for your platform from the Assets below
- Make it executable (Linux): `chmod +x ifo-*`
- Run directly or move to PATH

**From Source:**
```bash
git clone https://github.com/YOUR_USERNAME/ifo.git
cd ifo
cargo build --release
```

### Quick Start
```bash
# Docker
docker-compose up -d

# Or binary
ifo --dir ~/Downloads --dry-run  # Test first
ifo --dir ~/Downloads            # Run for real
```

### What's Included
- `ifo-linux-amd64` - Linux x86_64 static binary (works on Arch, CachyOS, Fedora, etc.)
- `ifo-linux-amd64-glibc` - Linux x86_64 glibc binary (for Ubuntu/Debian)
- `ifo-windows-amd64.exe` - Windows x86_64 binary
- Docker support
- Example config.toml

See [README](README.md) for full documentation.
