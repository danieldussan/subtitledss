# subtitledss

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-1.0.0-green.svg)
![CI](https://github.com/danieldussan/subtitledss/actions/workflows/ci.yml/badge.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)
![Whisper](https://img.shields.io/badge/whisper-cpp-orange.svg)

**Real-time subtitle overlay for Linux.** 100% offline, powered by whisper.cpp.

![subtitledss](src-tauri/icons/icon.png)

## Highlights

- **Real-time transcription** — speech-to-text as it happens, powered by whisper.cpp
- **Offline translation** — English ↔ Spanish via Marian MT (runs entirely on CPU)
- **Transparent overlay** — always-on-top subtitles with full appearance control
- **Zero cloud** — no data ever leaves your machine
- **Multiple models** — tiny (39 MB) to large-v3 (3.1 GB), GPU acceleration optional
- **History & search** — SQLite FTS5 full-text search across all transcriptions
- **Export** — SRT, VTT, TXT, JSON formats

---

## Installation

### Arch Linux (AUR)

```bash
yay -S subtitledss
# or
paru -S subtitledss
```

### AppImage (any distro)

```bash
# Download from https://github.com/danieldussan/subtitledss/releases
chmod +x subtitledss-*.AppImage
./subtitledss-*.AppImage
```

### Debian / Ubuntu

```bash
sudo dpkg -i subtitledss-*.deb
sudo apt-get install -f   # fix dependencies if needed
```

### From Source

**Prerequisites:**

| Distro          | Packages                                                                                                                      |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Arch Linux      | `sudo pacman -S rust bun pipewire libpipewire cmake`                                                                          |
| Ubuntu / Debian | `sudo apt install rustc cargo bun libpipewire-0.3-dev libasound2-dev cmake libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev` |
| Fedora          | `sudo dnf install rust cargo bun pipewire-devel alsa-lib-devel cmake glib2-devel gtk3-devel webkit2gtk4.1-devel`              |

**Build:**

```bash
git clone https://github.com/danieldussan/subtitledss.git
cd subtitledss
bun install
bun run tauri build
```

Binary: `src-tauri/target/release/subtitledss`

**GPU acceleration (optional):**

```bash
# NVIDIA (requires CUDA toolkit)
cd src-tauri && cargo build --release --features cuda

# Vulkan
cd src-tauri && cargo build --release --features vulkan
```

---

## Quick Start

1. Launch `subtitledss`
2. Go to **Settings** → download a Whisper model (start with **Base** — 142 MB)
3. Click **Load** on the downloaded model
4. Click the microphone button or press `Ctrl+Shift+S` to start capture
5. Subtitles appear in a transparent overlay on screen

---

## Configuration

Config file: `~/.config/subtitledss/config.toml`

### `[audio]`

| Field           | Type    | Default     | Description                                  |
| --------------- | ------- | ----------- | -------------------------------------------- |
| `source`        | string  | `"system"`  | Audio source                                 |
| `device`        | string  | `"default"` | Input device name                            |
| `sample_rate`   | integer | `16000`     | Sample rate in Hz                            |
| `vad_threshold` | float   | `0.005`     | Voice activity detection threshold (0.0–1.0) |

### `[whisper]`

| Field      | Type    | Default  | Description                                               |
| ---------- | ------- | -------- | --------------------------------------------------------- |
| `model`    | string  | `"base"` | Model name: `tiny`, `base`, `small`, `medium`, `large-v3` |
| `language` | string  | `"auto"` | Language code or `auto` for detection                     |
| `threads`  | integer | `4`      | CPU threads for inference                                 |
| `gpu`      | boolean | `false`  | Enable GPU acceleration (requires CUDA/Vulkan build)      |

### `[overlay]`

| Field                 | Type    | Default       | Description                             |
| --------------------- | ------- | ------------- | --------------------------------------- |
| `x`                   | integer | `100`         | Horizontal position (px)                |
| `y`                   | integer | `500`         | Vertical position (px)                  |
| `width`               | integer | `600`         | Window width (px)                       |
| `height`              | integer | `100`         | Window height (px)                      |
| `opacity`             | float   | `0.9`         | Background opacity (0.1–1.0)            |
| `always_on_top`       | boolean | `true`        | Keep overlay above other windows        |
| `click_through`       | boolean | `false`       | Allow mouse events to pass through      |
| `font_size`           | integer | `24`          | Font size (px)                          |
| `font_color`          | string  | `"#ffffff"`   | Font color (hex)                        |
| `background_color`    | string  | `"#00000080"` | Background color (hex + alpha)          |
| `auto_hide`           | boolean | `true`        | Hide when no speech detected            |
| `auto_hide_delay`     | integer | `5000`        | Delay before hiding (ms)                |
| `display_duration_ms` | integer | `10000`       | How long subtitles stay visible (ms)    |
| `fade_duration_ms`    | integer | `3000`        | Fade-out animation duration (ms)        |
| `max_visible_lines`   | integer | `4`           | Max subtitle lines shown at once        |
| `line_gap`            | integer | `4`           | Gap between lines (px)                  |
| `max_line_width`      | integer | `80`          | Max characters per line before wrapping |

### `[translation]`

| Field           | Type    | Default | Description                          |
| --------------- | ------- | ------- | ------------------------------------ |
| `enabled`       | boolean | `false` | Enable offline translation           |
| `source_lang`   | string  | `"en"`  | Source language (`en` or `es`)       |
| `target_lang`   | string  | `"es"`  | Target language (`en` or `es`)       |
| `show_original` | boolean | `true`  | Show original text below translation |

### `[shortcuts]`

| Field                | Type   | Default          | Description                 |
| -------------------- | ------ | ---------------- | --------------------------- |
| `toggle_capture`     | string | `"Ctrl+Shift+S"` | Start/stop audio capture    |
| `toggle_overlay`     | string | `"Ctrl+Shift+O"` | Show/hide overlay           |
| `toggle_translation` | string | `"Ctrl+Shift+T"` | Enable/disable translation  |
| `clear_history`      | string | `"Ctrl+Shift+H"` | Clear transcription history |

**Full example:**

```toml
[audio]
source = "system"
device = "default"
sample_rate = 16000
vad_threshold = 0.005

[whisper]
model = "base"
language = "auto"
threads = 4
gpu = false

[overlay]
x = 100
y = 500
width = 600
height = 100
opacity = 0.9
always_on_top = true
click_through = false
font_size = 24
font_color = "#ffffff"
background_color = "#00000080"
auto_hide = true
auto_hide_delay = 5000
display_duration_ms = 10000
fade_duration_ms = 3000
max_visible_lines = 4
line_gap = 4
max_line_width = 80

[translation]
enabled = false
source_lang = "en"
target_lang = "es"
show_original = true

[shortcuts]
toggle_capture = "Ctrl+Shift+S"
toggle_overlay = "Ctrl+Shift+O"
toggle_translation = "Ctrl+Shift+T"
clear_history = "Ctrl+Shift+H"
```

---

## Features

### Real-time Transcription

Audio is captured via PipeWire/CPAL, processed through a VAD (Voice Activity Detector), and transcribed in chunks by whisper.cpp. The pipeline runs asynchronously — transcription never blocks audio capture.

### Offline Translation

Built-in English ↔ Spanish translation using [Marian MT](https://github.com/huggingface/transformers) via the candle ML framework. Models (~300 MB each) are downloaded from HuggingFace and run entirely on CPU. No internet required after download.

| Pair              | Model                      |
| ----------------- | -------------------------- |
| English → Español | Helsinki-NLP/opus-mt-en-es |
| Español → English | Helsinki-NLP/opus-mt-es-en |

### Transparent Overlay

A borderless, always-on-top window renders subtitles with configurable font, colors, opacity, and timing. Supports auto-hide with configurable delay, fade animations, and click-through mode for seamless integration.

### Model Manager

Download, load, and delete Whisper models directly from the UI. Available models:

| Model    | Size   | Speed   | Accuracy |
| -------- | ------ | ------- | -------- |
| tiny     | 39 MB  | Fastest | Basic    |
| base     | 142 MB | Fast    | Good     |
| small    | 466 MB | Medium  | Better   |
| medium   | 1.5 GB | Slow    | Great    |
| large-v3 | 3.1 GB | Slowest | Best     |

### History & Search

All transcriptions are stored in a local SQLite database with FTS5 full-text search. Search across all past transcriptions instantly.

### Export

Export transcription history to:

- **SRT** — SubRip (widely supported by video players)
- **VTT** — WebVTT (web standard)
- **TXT** — Plain text
- **JSON** — Structured data with timestamps

### Keyboard Shortcuts & System Tray

| Shortcut       | Action                    |
| -------------- | ------------------------- |
| `Ctrl+Shift+S` | Toggle audio capture      |
| `Ctrl+Shift+O` | Toggle overlay visibility |
| `Ctrl+Shift+T` | Toggle translation        |
| `Ctrl+Shift+H` | Clear history             |

System tray provides quick access to start/stop capture, toggle overlay, show window, and quit.

---

## Development

### Tech Stack

| Layer         | Technology                    |
| ------------- | ----------------------------- |
| Backend       | Rust + Tauri 2                |
| Frontend      | React 19 + TypeScript         |
| Styling       | Tailwind CSS 4                |
| Build         | Vite 7                        |
| Linting       | oxlint (OXC)                  |
| Formatting    | oxfmt (OXC)                   |
| Transcription | whisper.cpp (whisper-rs 0.16) |
| Audio         | CPAL 0.18 (PipeWire)          |
| Translation   | candle + Marian MT            |
| Database      | SQLite + FTS5                 |

### Project Structure

```
subtitledss/
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── audio/            # CPAL capture, ring buffer
│   │   ├── whisper/          # whisper-rs engine, model downloader
│   │   ├── vad/              # Voice activity detector
│   │   ├── overlay/          # Overlay window manager
│   │   ├── translation/      # Marian MT engine + model manager
│   │   ├── settings/         # TOML config
│   │   ├── history/          # SQLite + FTS5
│   │   ├── models/           # Whisper model manager
│   │   ├── pipeline/         # Transcription pipeline
│   │   └── commands/         # Tauri IPC commands (21 total)
│   └── Cargo.toml
├── src/                      # React frontend
│   ├── components/
│   │   ├── Dashboard/        # Status cards, quick actions, live panel
│   │   ├── Settings/         # Audio, Whisper, Translation, Theme
│   │   ├── Overlay/          # Overlay settings + live preview
│   │   ├── History/          # History list with search
│   │   ├── ModelManager/     # Model download/load/delete
│   │   ├── Onboarding/       # First-run wizard
│   │   └── Layout/           # Sidebar, AppShell, routing
│   ├── hooks/                # useToast, useSettings, useOverlay
│   └── styles/               # Tailwind CSS theme
├── public/
│   └── overlay.html          # Overlay webview (vanilla JS)
├── ui-mockups/               # Design mockups
├── aur/                      # Arch Linux AUR packaging
└── .github/workflows/        # CI/CD
```

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
cargo test           # Run 93 unit tests
cargo build          # Build binary
```

---

## CI/CD

GitHub Actions runs **6 checks** on every push and PR to `main`:

| Job           | What it does                                            |
| ------------- | ------------------------------------------------------- |
| Lint & Format | `bun run lint` + `bun run fmt:check`                    |
| TypeCheck     | `bun run typecheck`                                     |
| Rust Clippy   | `cargo clippy --all-targets -- -D warnings`             |
| Rust Tests    | `cargo test` (93 tests)                                 |
| Rust Check    | `cargo check --all-targets`                             |
| Tauri Build   | Full `bun run tauri build` (requires all above to pass) |

---

## License

[MIT License](LICENSE) — Copyright (c) 2026 danieldussan
