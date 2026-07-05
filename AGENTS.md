# AGENTS.md

## Project Overview
**subtitledss** (package name: `subtitledss`) - Real-time subtitle overlay for Linux
- Open source, MIT license
- 100% offline, no cloud services
- Optimized for Arch Linux, Wayland, Hyprland
- Future: Windows/macOS compatibility

## Tech Stack
- **Backend**: Rust with Tauri 2
- **Frontend**: React 19 + TypeScript + Bun
- **Styling**: Tailwind CSS 4
- **Build**: Vite 7
- **AI Engine**: whisper.cpp (whisper-rs 0.16)
- **Audio**: CPAL 0.18 (PipeWire)
- **Database**: SQLite + FTS5
- **Linting**: oxlint (OXC)
- **Formatting**: oxfmt (OXC)

## Commands

### Frontend
```bash
bun run dev          # Start dev server
bun run build        # Build frontend
bun run lint         # Run oxlint
bun run lint:fix     # Auto-fix lint issues
bun run fmt          # Format with oxfmt
bun run fmt:check    # Check formatting
bun run typecheck    # TypeScript check
```

### Tauri (Rust)
```bash
cd src-tauri
cargo check          # Check compilation
cargo test           # Run tests
cargo build          # Build binary
```

## Code Conventions

### Rust
- Use `Arc<Mutex<T>>` for shared state
- Avoid holding `MutexGuard` across `.await` points
- Clone data before dropping locks
- Import `Emitter` and `Manager` traits from `tauri::Manager` / `tauri::Emitter`
- Use `anyhow` for error handling in commands
- Config structs derive `PartialEq` for test assertions
- Tests use `AtomicU64` counters for unique temp directories

### TypeScript/React
- Functional components with hooks
- Error handling with try/catch and user-facing toast notifications
- Dark theme with slate-based neutrals
- Design system: `.card`, `.btn`, `.input`, `.select`, `.tab-bar`, `.toggle-switch`
- Framer Motion for animations
- lucide-react for icons
- `@tauri-apps/api/core` for IPC calls (`invoke`)

### File Structure
```
src-tauri/src/
├── audio/          # CPAL capture, ring buffer
├── whisper/        # whisper-rs engine, model downloader
├── vad/            # Voice activity detector
├── overlay/        # Overlay manager
├── settings/       # TOML config
├── history/        # SQLite + FTS5
├── models/         # Model manager
├── pipeline/       # Transcription pipeline
├── commands/       # Tauri IPC commands
└── lib.rs          # Main entry, wires everything

src/
├── components/
│   ├── Settings/   # Audio, Whisper, Theme settings
│   ├── History/    # History list with search
│   ├── ModelManager/ # Model download/load/delete
│   └── Overlay/    # Overlay window
├── hooks/          # useToast, useSettings, useOverlay, useTranscription
└── App.tsx         # Main app with tabs
```

## Testing
- **Rust**: 76 unit tests (buffer: 22, VAD: 16, config: 14, history: 17)
- **TypeScript**: `bun run typecheck` (no runtime tests yet)
- Run `cargo test` before commits
- Run `bun run typecheck` before commits

## CI/CD
- GitHub Actions: `.github/workflows/ci.yml`
- Jobs: lint, typecheck, rust-test, rust-check, frontend-build, tauri-build
- All checks must pass before merge

## Performance Targets
- Latency <500 ms with small models
- Low RAM/CPU consumption
- Ring buffer: 4096 samples (~256ms at 16kHz)

## SpecKit
- Documentation in `.specify/`
- Constitution: principles, tech stack, roadmap
- Features: 4 phases (core transcription, stability, UX, functionality)
- Plan: three-phase timeline with dependencies
- Tasks: full task board with checkboxes
