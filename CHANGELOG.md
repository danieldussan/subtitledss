# Changelog

All notable changes to subtitledss will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-beta] - 2026-07-05

### Added

#### Core
- Real-time transcription using whisper.cpp (whisper-rs 0.16)
- System audio capture via PipeWire/CPAL
- Audio resampling (48kHz → 16kHz) with linear interpolation
- Stereo to mono downmix
- Voice Activity Detection (VAD) with configurable threshold

#### Overlay
- Transparent subtitle overlay window
- Always-on-top mode
- Draggable overlay position
- Auto-hide when no speech detected
- Customizable font size, colors, opacity

#### UI
- System tray with menu (start/stop, show overlay, quit)
- Global keyboard shortcuts (Ctrl+Shift+S, Ctrl+Shift+O)
- Real-time audio level meter
- Model manager (download, load, delete)
- Settings panel (audio, whisper, overlay, theme)
- History with full-text search (SQLite FTS5)
- Export to SRT, VTT, TXT, JSON

#### Performance
- Buffer overflow protection (drops old audio when falling behind)
- Speed ratio logging for performance monitoring
- Pipeline status events for frontend feedback

#### Testing
- 83 unit tests (buffer: 25, VAD: 16, config: 17, history: 17, export: 4, pipeline: 4)
- CI/CD pipeline with lint, typecheck, rust-test, frontend-build

#### GPU
- Optional CUDA acceleration (build with `--features cuda`)
- Optional Vulkan acceleration (build with `--features vulkan`)
- Auto-detection at build time

#### Packaging
- AUR package for Arch Linux
- AppImage for universal Linux
- .deb package for Debian/Ubuntu
- Raw binary archive

### Changed
- Pipeline redesign: real-time chunked transcription (3s chunks) instead of VAD-gated
- Buffer uses `take()` instead of `drain()` for consistent chunk sizes
- Pipeline reads config (threads, gpu, language) instead of hardcoding

### Fixed
- Event name mismatch between overlay and pipeline (`subtitle-update` → `transcription`)
- Buffer drain taking all samples instead of exact chunk size
- Test isolation with unique temp directories per test (AtomicU64)
- MutexGuard Send issues in async pipeline code

### Known Issues
- GPU detection requires CUDA toolkit installed at build time
- Overlay position not saved to config on drag end
- Virtualized history list not yet implemented (react-window)
