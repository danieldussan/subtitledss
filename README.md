# subtitledss

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0--beta-green.svg)
![CI](https://github.com/danieldussan/subtitledss/actions/workflows/ci.yml/badge.svg)

Real-time subtitle overlay for Linux, powered by Whisper.

![subtitledss](src-tauri/icons/icon.png)

## Features

- **Real-time transcription** using whisper.cpp
- **System audio capture** via PipeWire/CPAL
- **Transparent overlay** with customizable appearance
- **Offline processing** - no data leaves your machine
- **Multiple models** - tiny to large-v3
- **History** with full-text search (SQLite FTS5)
- **GPU acceleration** support (CUDA/Vulkan, optional)
- **Export** to SRT, VTT, TXT, JSON

## Installation

### Arch Linux (AUR)

```bash
# Using yay
yay -S subtitledss

# Using paru
paru -S subtitledss
```

### AppImage (any distro)

1. Download the latest `.AppImage` from [Releases](https://github.com/danieldussan/subtitledss/releases)
2. Make it executable:
   ```bash
   chmod +x subtitledss-*.AppImage
   ```
3. Run it:
   ```bash
   ./subtitledss-*.AppImage
   ```

### Debian/Ubuntu

```bash
# Download the .deb from Releases
sudo dpkg -i subtitledss-*.deb
sudo apt-get install -f  # Fix dependencies if needed
```

### Fedora/RHEL

```bash
# Build from source (see below)
```

### From Source

#### Prerequisites

**Arch Linux:**
```bash
sudo pacman -S rust bun pipewire libpipewire cmake
```

**Ubuntu/Debian:**
```bash
sudo apt install rustc cargo bun libpipewire-0.3-dev libasound2-dev cmake libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev
```

**Fedora:**
```bash
sudo dnf install rust cargo bun pipewire-devel alsa-lib-devel cmake glib2-devel gtk3-devel webkit2gtk4.1-devel
```

#### Build

```bash
# Clone the repository
git clone https://github.com/danieldussan/subtitledss.git
cd subtitledss

# Install frontend dependencies
bun install

# Build for production
bun run tauri build
```

The binary will be at `src-tauri/target/release/subtitledss`.

#### Build with GPU Acceleration

**NVIDIA (CUDA):**
```bash
# Requires CUDA toolkit installed
cd src-tauri
cargo build --release --features cuda
```

**Vulkan:**
```bash
cd src-tauri
cargo build --release --features vulkan
```

## Usage

1. **Start the app**: `subtitledss` or `bun run tauri dev`
2. **Download a model**: Go to Model Manager tab, download a model (start with Tiny or Base)
3. **Load the model**: Click "Load" on the downloaded model
4. **Start capture**: Click the microphone button in the header
5. **View subtitles**: Subtitles appear in a transparent overlay window

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+S` | Toggle audio capture |
| `Ctrl+Shift+O` | Toggle overlay visibility |

### Overlay Controls

- **Drag**: Move the overlay by dragging
- **Auto-hide**: Overlay hides when no speech is detected (configurable delay)
- **Toggle**: Use keyboard shortcut or tray menu

### Settings

- **Audio**: Select input device and sample rate
- **Whisper**: Configure model, language, threads, GPU
- **Overlay**: Customize appearance (font, colors, opacity, position)

## Configuration

Config file is stored at `~/.config/subtitledss/config.toml`:

```toml
[audio]
source = "system"
device = "default"
sample_rate = 16000
vad_threshold = 0.005

[whisper]
model = "base"
language = "es"
threads = 4
gpu = false

[overlay]
x = 100
y = 500
width = 600
height = 100
opacity = 0.9
always_on_top = true
font_size = 24
font_color = "#ffffff"
background_color = "#00000080"
```

## Development

### Commands

```bash
# Frontend
bun run dev          # Start dev server
bun run build        # Build frontend
bun run lint         # Run oxlint
bun run lint:fix     # Auto-fix lint issues
bun run fmt          # Format with oxfmt
bun run fmt:check    # Check formatting
bun run typecheck    # TypeScript check

# Tauri (Rust)
cd src-tauri
cargo check          # Check compilation
cargo test           # Run tests
cargo build          # Build binary
```

### Project Structure

```
subtitledss/
├── src-tauri/        # Rust backend
│   ├── src/
│   │   ├── audio/    # CPAL capture, ring buffer
│   │   ├── whisper/  # whisper-rs engine
│   │   ├── vad/      # Voice activity detector
│   │   ├── overlay/  # Overlay manager
│   │   ├── settings/ # TOML config
│   │   ├── history/  # SQLite + FTS5
│   │   ├── models/   # Model manager
│   │   ├── pipeline/ # Transcription pipeline
│   │   └── commands/ # Tauri IPC commands
│   └── Cargo.toml
├── src/              # React frontend
│   ├── components/   # UI components
│   ├── hooks/        # Custom hooks
│   └── styles/       # Tailwind CSS
├── aur/              # Arch Linux package
├── config/           # Default config
└── .specify/         # SpecKit documentation
```

### Testing

```bash
# Run Rust tests
cd src-tauri
cargo test

# Run TypeScript check
bun run typecheck
```

## Contributing

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
