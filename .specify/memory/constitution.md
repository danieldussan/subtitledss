# subtitledss Constitution

## Core Principles

### I. Offline-First
All processing happens locally. No audio, transcription, or user data ever leaves the machine. No network requests except for model downloads (with explicit user consent).

### II. Real-Time Performance
Target latency <500ms for Tiny/Base models. The audio → VAD → Whisper → overlay pipeline must be non-blocking and responsive. UI must maintain 60fps.

### III. Modular Architecture
Each subsystem (audio, whisper, VAD, overlay, history, settings) is an independent Rust module with clear boundaries. No circular dependencies. Frontend components are self-contained with their own hooks.

### IV. Linux-Native
Optimized for Arch Linux, Wayland, and Hyprland. PipeWire is the primary audio backend. The app must feel native, not cross-platform.

### V. Test-First
Every module must have unit tests. Integration tests for the audio pipeline. Frontend components must be testable in isolation.

## Technology Stack (Non-Negotiable)

| Layer | Technology | Reason |
|-------|-----------|--------|
| Shell | Tauri 2 | Native, small binary |
| Frontend | React 19 + TypeScript | Fast dev, strong typing |
| Styling | Tailwind CSS 4 | Utility-first, consistent |
| Backend | Rust | Safety, performance |
| Audio | CPAL + PipeWire | Low-latency Linux capture |
| STT | whisper-rs (whisper.cpp) | Local, fast, multi-model |
| DB | SQLite + FTS5 | Full-text search for history |
| Config | TOML (serde) | Human-readable |

## UI/UX Standards

### Design System
- **Dark theme only** (no light mode in v1)
- **Color palette**: Slate-based neutrals with blue-500 accent
- **Typography**: Inter font family, 14px base
- **Spacing**: 4px grid system (4, 8, 12, 16, 20, 24, 32, 40, 48)
- **Border radius**: 8px default, 12px for cards, 16px for modals
- **Shadows**: Subtle, layered (not heavy drop shadows)

### Contrast Requirements
- Text on background: minimum 4.5:1 ratio (WCAG AA)
- Interactive elements: minimum 3:1 ratio against adjacent colors
- Focus indicators: visible, 2px outline with 2px offset

### Interaction Patterns
- All clickable elements have hover/active states
- Loading states for async operations
- Error messages are inline, not blocking
- Confirmation dialogs for destructive actions

## Performance Standards

| Metric | Target |
|--------|--------|
| Cold start | <3s |
| Audio capture start | <500ms |
| Transcription latency | <500ms (Tiny/Base) |
| UI FPS | 60fps |
| RAM idle | <150MB |
| RAM with Tiny | <400MB |
| RAM with Base | <600MB |
| CPU idle | <2% |

## Security

- No remote code execution
- No telemetry without consent
- No secrets in code
- Models downloaded over HTTPS with checksum verification
- Config files are user-writable only

## Roadmap

### Phase 1: Stability (MVP Release)
- Unit tests for core modules
- CI/CD pipeline (GitHub Actions)
- Toast notification system
- Error handling in UI
- Project documentation (AGENTS.md, README, CONTRIBUTING)

### Phase 2: UX Polish
- System tray integration
- Global keyboard shortcuts
- Model state visibility
- VAD threshold configuration
- Status feedback improvements

### Phase 3: Core Functionality
- Export transcriptions (SRT/VTT/TXT/JSON)
- Scrollable history with pagination
- Draggable overlay
- Local translation (NLLB-200)
- History search improvements

## Governance

This constitution is the source of truth for all development decisions. All PRs must verify compliance with these principles. Complexity must be justified by a clear user need.

**Version**: 1.1.0 | **Ratified**: 2026-07-05 | **Last Amended**: 2026-07-05
