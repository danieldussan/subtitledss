# Feature: Core Transcription Pipeline

## Overview
Real-time speech-to-text transcription using Whisper, with audio capture via PipeWire, VAD for silence detection, and a transparent overlay for displaying subtitles.

## User Stories

### US-1: Start/Stop Transcription
**As a** user  
**I want to** start and stop transcription with a single click or keyboard shortcut  
**So that** I can control when my audio is being processed

**Acceptance Criteria:**
- [ ] "Start Capture" button begins audio capture and transcription
- [ ] "Stop Capture" button stops audio capture and transcription
- [ ] Keyboard shortcut Ctrl+Shift+S toggles capture
- [ ] Visual indicator shows capture state (green=active, gray=inactive)
- [ ] Audio device is released when capture stops

### US-2: View Transcription in Real-Time
**As a** user  
**I want to** see transcribed text appear on screen as I speak  
**So that** I can follow along with live audio

**Acceptance Criteria:**
- [ ] Text appears within 500ms of speech ending
- [ ] Overlay is transparent and always on top
- [ ] Overlay is draggable
- [ ] Overlay auto-hides after 5 seconds of silence
- [ ] Text is readable against any background (high contrast)

### US-3: Select Audio Source
**As a** user  
**I want to** choose between system audio, microphone, or both  
**So that** I can transcribe different audio sources

**Acceptance Criteria:**
- [ ] Settings panel shows audio source options
- [ ] Can switch between system/mic/both without restart
- [ ] Device list shows all available input devices
- [ ] Default device is pre-selected

### US-4: Select Whisper Model
**As a** user  
**I want to** choose which Whisper model to use  
**So that** I can balance speed vs accuracy

**Acceptance Criteria:**
- [ ] Model selection shows Tiny/Base/Small/Medium/Large-v3
- [ ] Shows model size (MB) for each option
- [ ] Model is downloaded on first use if not present
- [ ] Download progress is shown
- [ ] Can delete downloaded models to free space

### US-5: Select Language
**As a** user  
**I want to** set the transcription language or auto-detect  
**So that** transcription is accurate for my language

**Acceptance Criteria:**
- [ ] Language dropdown with major languages
- [ ] "Auto-detect" option
- [ ] Language setting persists across restarts

### US-6: View History
**As a** user  
**I want to** see past transcriptions with timestamps  
**So that** I can review what was said

**Acceptance Criteria:**
- [ ] History list shows timestamp, language, text
- [ ] Can search history by text content
- [ ] Can clear history
- [ ] History is persisted in SQLite

### US-7: Configure Overlay Appearance
**As a** user  
**I want to** customize the overlay look (font size, colors, opacity)  
**So that** it fits my workflow and display setup

**Acceptance Criteria:**
- [ ] Font size slider (12-72px)
- [ ] Font color picker
- [ ] Background color picker (with transparency)
- [ ] Opacity slider
- [ ] Changes apply immediately

## Technical Architecture

```
┌─────────────────────────────────────────────┐
│              Tauri 2 Shell                   │
├─────────────────────────────────────────────┤
│  Main Window (Settings / History / Models)  │
│  Overlay Window (transparent, always-on-top)│
└─────────────────────────────────────────────┘
                     │
          IPC (Commands / Events)
                     │
┌─────────────────────────────────────────────┐
│              Rust Backend                    │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────┐   ┌─────────┐   ┌──────────┐ │
│  │  CPAL   │──▶│   VAD   │──▶│  Ring    │ │
│  │ Capture │   │Detector │   │  Buffer  │ │
│  └─────────┘   └─────────┘   └────┬─────┘ │
│                                    │       │
│  ┌─────────────────────────────────▼─────┐ │
│  │         Whisper Engine                │ │
│  │  (whisper-rs, model loaded once)      │ │
│  └─────────────────────────────────┬─────┘ │
│                                    │       │
│  ┌─────────────────────────────────▼─────┐ │
│  │      Post-processing                  │ │
│  │  (timestamps, text cleanup)           │ │
│  └─────────────────────────────────┬─────┘ │
│                                    │       │
│  ┌─────────────────────────────────▼─────┐ │
│  │      Overlay Manager                  │ │
│  │  (position, opacity, click-through)   │ │
│  └───────────────────────────────────────┘ │
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Settings │  │ History  │  │  Models  │ │
│  │ (TOML)   │  │ (SQLite) │  │ Manager  │ │
│  └──────────┘  └──────────┘  └──────────┘ │
└─────────────────────────────────────────────┘
```

## Data Flow

```
Audio Source (PipeWire)
    │
    ▼
CPAL Capture (f32 samples, 16kHz mono)
    │
    ▼
Ring Buffer (circular, 30s capacity)
    │
    ├──▶ VAD Check (energy threshold)
    │       │
    │       ▼
    │    Is Voice?
    │       │
    │       ├── No → Discard, continue
    │       │
    │       └── Yes → Collect segment
    │               │
    │               ▼
    │           Whisper Transcribe
    │               │
    │               ▼
    │           Post-process (clean text)
    │               │
    │               ▼
    │           IPC Event → Frontend
    │               │
    │               ▼
    │           Overlay Update
    │
    └──▶ History Save (SQLite)
```

## File Structure

```
src-tauri/src/
├── audio/
│   ├── mod.rs
│   ├── capture.rs      # CPAL audio capture
│   ├── buffer.rs       # Ring buffer
│   └── device.rs       # Device enumeration
├── whisper/
│   ├── mod.rs
│   ├── engine.rs       # whisper-rs wrapper
│   ├── model.rs        # Model loading/management
│   └── params.rs       # Transcription params
├── vad/
│   ├── mod.rs
│   └── detector.rs     # Voice activity detection
├── overlay/
│   ├── mod.rs
│   └── manager.rs      # Overlay window control
├── settings/
│   ├── mod.rs
│   └── config.rs       # TOML config handling
├── history/
│   ├── mod.rs
│   ├── db.rs           # SQLite operations
│   └── search.rs       # FTS5 search
├── models/
│   ├── mod.rs
│   ├── downloader.rs   # Model download
│   └── manager.rs      # Model lifecycle
├── commands/
│   ├── mod.rs
│   ├── transcription.rs # IPC commands
│   ├── settings.rs
│   └── history.rs
├── lib.rs
└── main.rs

src/
├── components/
│   ├── Overlay/
│   │   ├── OverlayWindow.tsx
│   │   └── SubtitleText.tsx
│   ├── Settings/
│   │   ├── SettingsPanel.tsx
│   │   ├── AudioSettings.tsx
│   │   ├── WhisperSettings.tsx
│   │   └── ThemeSettings.tsx
│   ├── History/
│   │   └── HistoryList.tsx
│   └── ModelManager/
│       └── ModelList.tsx
├── hooks/
│   ├── useTranscription.ts
│   ├── useSettings.ts
│   └── useOverlay.ts
├── App.tsx
├── main.tsx
└── styles/
    └── globals.css
```

## IPC Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `start_capture` | - | `Result<String>` | Start audio capture |
| `stop_capture` | - | `Result<String>` | Stop audio capture |
| `transcribe_audio` | `{audio_data, language, threads}` | `TranscriptionResult` | Transcribe audio chunk |
| `get_config` | - | `AppConfig` | Get current config |
| `save_config` | `{config}` | `Result<()>` | Save config |
| `list_audio_devices` | - | `Vec<String>` | List input devices |
| `get_history` | `{limit}` | `Vec<HistoryEntry>` | Get history |
| `search_history` | `{query, limit}` | `Vec<HistoryEntry>` | Search history |
| `clear_history` | - | `Result<()>` | Clear all history |
| `download_model` | `{model_name}` | `Result<PathBuf>` | Download model |
| `delete_model` | `{model_name}` | `Result<()>` | Delete model |
| `list_models` | - | `Vec<ModelInfo>` | List available models |

## Testing Strategy

### Unit Tests
- `audio/buffer.rs`: Ring buffer operations
- `vad/detector.rs`: Voice detection logic
- `whisper/engine.rs`: Transcription with mock audio
- `history/db.rs`: CRUD operations, FTS search
- `settings/config.rs`: Config load/save/defaults

### Integration Tests
- Audio capture → buffer → VAD pipeline
- Whisper model loading and transcription
- Config persistence across restarts

### E2E Tests (Manual)
- Start capture → speak → see text in overlay
- Change settings → verify they persist
- Download model → transcribe → verify output
