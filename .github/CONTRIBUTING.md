# Contributing to subtitledss

Thank you for your interest in contributing! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Linux (Arch Linux recommended)
- PipeWire audio server
- Rust 1.70+
- Bun

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/danieldussan/subtitledss.git
   cd subtitledss
   ```
3. Install dependencies:
   ```bash
   bun install
   ```
4. Start development:
   ```bash
   bun run tauri dev
   ```

## Development Workflow

### Code Style

#### Rust
- Use `Arc<Mutex<T>>` for shared state
- Avoid holding `MutexGuard` across `.await` points
- Clone data before dropping locks
- Import `Emitter` and `Manager` traits from `tauri::Manager` / `tauri::Emitter`
- Use `anyhow` for error handling in commands
- Config structs derive `PartialEq` for test assertions
- Tests use `AtomicU64` counters for unique temp directories

#### TypeScript/React
- Functional components with hooks
- Error handling with try/catch and user-facing toast notifications
- Dark theme with slate-based neutrals
- Design system: `.card`, `.btn`, `.input`, `.select`, `.tab-bar`, `.toggle-switch`
- Framer Motion for animations
- lucide-react for icons
- `@tauri-apps/api/core` for IPC calls (`invoke`)

### Linting and Formatting

We use OXC tools for linting and formatting:

```bash
# Lint
bun run lint          # Check for issues
bun run lint:fix      # Auto-fix issues

# Format
bun run fmt           # Format code
bun run fmt:check     # Check formatting
```

### Testing

```bash
# Run Rust tests
cd src-tauri
cargo test

# Run TypeScript check
bun run typecheck
```

### Commit Messages

- Use conventional commits: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `chore:`
- Keep commits focused and atomic
- Reference issues when applicable

### Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Run tests and linting:
   ```bash
   bun run lint
   bun run fmt:check
   bun run typecheck
   cd src-tauri && cargo test
   ```
4. Commit your changes with a clear message
5. Push to your fork
6. Create a pull request

### Code Review

- All PRs require review before merge
- Address feedback promptly
- Keep PRs focused and reasonably sized

## Architecture

### Backend (Rust)

- **Audio**: CPAL capture, ring buffer, device enumeration
- **Whisper**: whisper-rs engine, model downloader
- **VAD**: Voice activity detector
- **Overlay**: Overlay window manager
- **Settings**: TOML configuration
- **History**: SQLite + FTS5 search
- **Models**: Model manager
- **Pipeline**: Transcription pipeline (buffer → VAD → Whisper → events)
- **Commands**: Tauri IPC commands

### Frontend (React/TypeScript)

- **Components**: Settings, History, ModelManager, Overlay
- **Hooks**: useToast, useSettings, useOverlay, useTranscription
- **Styling**: Tailwind CSS 4 with custom design system

### IPC Commands

- `start_capture` / `stop_capture`: Audio capture control
- `download_model` / `delete_model` / `load_model`: Model management
- `get_history` / `search_history` / `clear_history`: History management
- `get_config` / `save_config`: Settings management

## Performance Targets

- Latency <500 ms with small models
- Low RAM/CPU consumption
- Ring buffer: 4096 samples (~256ms at 16kHz)

## Roadmap

See `.specify/plan.md` for the three-phase roadmap:
- **Phase 1**: Stability (tests, CI/CD, error handling)
- **Phase 2**: UX (system tray, global shortcuts, VAD config)
- **Phase 3**: Functionality (export, translation, overlay drag)

## Questions?

Open an issue or start a discussion on GitHub.
