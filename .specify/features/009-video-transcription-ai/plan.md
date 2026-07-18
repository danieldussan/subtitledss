# Implementation Plan: 009-video-transcription-ai

**Feature**: Video Transcription, AI Chat & Speaker Diarization
**Branch**: `009-video-transcription-ai`
**Created**: 2026-07-17
**Status**: Ready

---

## Technical Context

| Item | Value |
|------|-------|
| Backend | Rust + Tauri 2 |
| Frontend | React 19 + TypeScript + Bun |
| Styling | Tailwind CSS 4 (design tokens in globals.css) |
| Animation | framer-motion |
| Icons | lucide-react |
| State | React hooks + `invoke()` IPC |
| Build | Vite 7, oxlint, oxfmt |
| Database | SQLite + FTS5 (existing) |
| Video Processing | FFmpeg via `std::process::Command` |
| Speaker Diarization | `speakrs` crate (ONNX-based, pyannote-level accuracy) |
| AI Providers | OpenAI-compatible `/v1/chat/completions` via `reqwest` |

---

## Constitution Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| Offline-First | ✅ PASS | Video transcription uses local Whisper. AI providers can be local (Ollama, LM Studio). DeepSeek is optional cloud. |
| Real-Time Performance | ✅ PASS | Video transcription is batch (not real-time). AI streaming uses token-by-token delivery. |
| Modular Architecture | ✅ PASS | New modules (`video/`, `ai/`, `diarization/`) are self-contained with clear interfaces. |
| Linux-Native | ✅ PASS | FFmpeg is standard on Arch Linux. speakrs runs on CPU without GPU. |
| Test-First | ⚠️ CONDITIONAL | Plan includes unit tests for new Rust modules; frontend verification via typecheck. |

---

## Project Structure

### New Files

```text
src-tauri/src/
├── ai/
│   ├── mod.rs                    # Module declarations
│   ├── provider.rs               # AiProvider trait + OpenAiCompatibleProvider
│   ├── config.rs                 # AiProviderType, AiConfig, ChatMessage
│   └── commands.rs               # Tauri commands: ai_summarize, ai_chat, etc.
├── video/
│   ├── mod.rs                    # Module declarations
│   └── processor.rs              # FFmpeg audio extraction, metadata
├── diarization/
│   ├── mod.rs                    # Module declarations
│   └── engine.rs                 # speakrs wrapper
└── commands/
    └── video_transcription.rs    # transcribe_video, list/del video transcriptions

src/components/VideoTranscription/
├── VideoTranscriptionPage.tsx    # Main page layout
├── VideoPicker.tsx               # File selection + metadata display
├── TranscriptionViewer.tsx       # Segment list with timestamps + speakers
├── AiPanel.tsx                   # Summary + Chat combined panel
├── AiChatMessage.tsx             # Individual chat message bubble
├── ExportMenu.tsx                # Export format picker dropdown
└── ProgressIndicator.tsx         # Multi-step progress display

src/components/Settings/
└── AiSettings.tsx                # AI provider configuration panel

src/hooks/
└── useVideoTranscription.ts      # State management hook for video page
```

### Modified Files

```text
src-tauri/src/
├── lib.rs                        # Add new modules, manage states, register commands
├── main.rs                       # No changes
├── settings/config.rs            # Add AiSettingsConfig to AppConfig
├── commands/
│   └── export.rs                 # Add ASS/SSA format + to_ass() function
└── history/
    └── db.rs                     # Add video_transcriptions table

src/components/Layout/
├── types.ts                      # Add "video" to Section type
├── SectionRouter.tsx             # Add video case
└── Sidebar.tsx                   # Add Video nav item (no changes needed, auto from SECTION_ITEMS)

src/hooks/
└── useSettings.ts                # Add ai section to AppConfig interface
```

---

## Phase 1: Video Processing — FFmpeg Audio Extraction

**Goal**: Extract audio from video files using FFmpeg system command.

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/video/mod.rs` | Module declarations |
| `src-tauri/src/video/processor.rs` | `VideoProcessor` with extract_audio, get_duration, get_metadata |

### Implementation

```rust
// video/processor.rs
pub struct VideoProcessor;

impl VideoProcessor {
    pub async fn extract_audio(video_path: &Path) -> anyhow::Result<PathBuf>;
    pub async fn get_duration(video_path: &Path) -> anyhow::Result<f64>;
    pub async fn get_metadata(video_path: &Path) -> anyhow::Result<VideoMetadata>;
}
```

- Uses `tokio::process::Command` for async FFmpeg execution
- Outputs 16kHz mono WAV (matches Whisper input requirements)
- Cleans up temp WAV file after transcription
- Returns `VideoMetadata` with format, duration, codec info

### Tests

- `test_extract_audio_creates_wav` — verify WAV file exists and is valid
- `test_get_duration_returns_positive` — verify duration > 0
- `test_extract_audio_unsupported_format` — verify error on corrupt file

---

## Phase 2: AI Provider System

**Goal**: Create a unified AI provider abstraction supporting Ollama, LM Studio, and DeepSeek.

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/ai/mod.rs` | Module declarations |
| `src-tauri/src/ai/config.rs` | `AiProviderType` enum, `AiConfig`, `ChatMessage`, `ChatRole` |
| `src-tauri/src/ai/provider.rs` | `AiProvider` trait + `OpenAiCompatibleProvider` implementation |
| `src-tauri/src/ai/commands.rs` | Tauri commands for AI operations |

### Implementation

```rust
// ai/config.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiProviderType {
    Ollama,
    LmStudio,
    DeepSeek,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: AiProviderType,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

// ai/provider.rs
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, system_prompt: &str, messages: &[ChatMessage]) -> anyhow::Result<String>;
    async fn chat_stream(&self, system_prompt: &str, messages: &[ChatMessage]) -> anyhow::Result<mpsc::Receiver<String>>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}

pub struct OpenAiCompatibleProvider {
    config: AiConfig,
    client: reqwest::Client,
}
```

- All providers share OpenAI-compatible API format
- `chat()` for summaries (non-streaming)
- `chat_stream()` for chat (token-by-token streaming)
- `reqwest` is already a dependency — no new crates needed

### Tests

- `test_provider_config_defaults` — verify default URLs for each provider
- `test_chat_message_serialization` — verify JSON format matches OpenAI API

---

## Phase 3: Speaker Diarization

**Goal**: Integrate `speakrs` crate for speaker identification.

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/diarization/mod.rs` | Module declarations |
| `src-tauri/src/diarization/engine.rs` | `DiarizationEngine` wrapping `speakrs::OfflineDiarizer` |

### Implementation

```rust
// diarization/engine.rs
pub struct DiarizationEngine {
    // Lazy initialization — models downloaded on first use
}

impl DiarizationEngine {
    pub async fn new() -> anyhow::Result<Self>;
    pub async fn diarize(&self, audio_path: &Path) -> anyhow::Result<Vec<SpeakerTurn>>;
}

pub struct SpeakerTurn {
    pub start: f64,
    pub end: f64,
    pub speaker: String,
}
```

- Uses `speakrs` with `cpu` feature (no GPU required)
- Models auto-download on first use via `speakrs` ModelManager
- Single-speaker guard prevents hallucinated clusters
- Falls back gracefully if diarization fails

### Dependency

```toml
# src-tauri/Cargo.toml
speakrs = { version = "0.4", features = ["cpu", "download"] }
```

### Tests

- `test_speaker_turn_struct` — verify struct fields
- `test_diarization_engine_new` — verify initialization (may need network for model download)

---

## Phase 4: Tauri Commands — Video Transcription

**Goal**: Wire together video processing, Whisper, and diarization into Tauri commands.

### Files to Create

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/video_transcription.rs` | Main commands for video transcription workflow |

### Commands

```rust
#[tauri::command]
pub async fn transcribe_video(
    video_path: String,
    language: Option<String>,
    enable_diarization: bool,
    state: State<'_, VideoTranscriptionState>,
) -> Result<VideoTranscriptionResult, String>;

#[tauri::command]
pub async fn list_video_transcriptions(
    limit: Option<i64>,
    state: State<'_, VideoTranscriptionState>,
) -> Result<Vec<VideoTranscriptionEntry>, String>;

#[tauri::command]
pub async fn delete_video_transcription(
    id: i64,
    state: State<'_, VideoTranscriptionState>,
) -> Result<(), String>;

#[tauri::command]
pub async fn export_video_transcription(
    id: i64,
    format: String,
    path: String,
    state: State<'_, VideoTranscriptionState>,
) -> Result<(), String>;
```

### Database Schema

```sql
CREATE TABLE IF NOT EXISTS video_transcriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    video_path TEXT NOT NULL,
    video_name TEXT NOT NULL,
    duration_seconds REAL,
    language TEXT NOT NULL,
    full_text TEXT NOT NULL,
    summary TEXT,
    segments JSON NOT NULL,
    created_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS video_fts USING fts5(
    full_text,
    summary,
    content=video_transcriptions,
    content_rowid=id
);
```

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/src/lib.rs` | Add `mod video`, `mod ai`, `mod diarization`. Manage `VideoTranscriptionState`, `AiState`. Register new commands. |
| `src-tauri/src/history/db.rs` | Add `create_video_transcriptions_table()` method |
| `src-tauri/src/settings/config.rs` | Add `AiSettingsConfig` to `AppConfig` |

---

## Phase 5: Export — ASS/SSA Format

**Goal**: Add ASS/SSA subtitle export format.

### Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/commands/export.rs` | Add `to_ass()` function, add `ExportFormat::Ass` variant |

### Implementation

```rust
pub fn to_ass(entries: &[ExportEntry]) -> String {
    // [Script Info] header
    // [V4+ Styles] with Default style (Arial, white, outline)
    // [Events] with Dialogue lines: Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    // Timestamps in H:MM:SS.cc format (ASS uses centiseconds)
}
```

### Tests

- `test_ass_format_header` — verify Script Info section
- `test_ass_format_dialogue` — verify Dialogue line format
- `test_ass_format_timestamps` — verify H:MM:SS.cc format

---

## Phase 6: Frontend — Video Transcription Page

**Goal**: Build the complete UI for video transcription workflow.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/VideoTranscription/VideoTranscriptionPage.tsx` | Main page with 3-section layout |
| `src/components/VideoTranscription/VideoPicker.tsx` | File selection, metadata display, transcribe button |
| `src/components/VideoTranscription/TranscriptionViewer.tsx` | Scrollable segment list with timestamps |
| `src/components/VideoTranscription/ProgressIndicator.tsx` | Multi-step progress (extract → transcribe → process) |
| `src/components/VideoTranscription/ExportMenu.tsx` | Dropdown with format options |
| `src/hooks/useVideoTranscription.ts` | State management hook |

### Files to Modify

| File | Changes |
|------|---------|
| `src/components/Layout/types.ts` | Add `"video"` to `Section` type, add to `SECTION_ITEMS` |
| `src/components/Layout/SectionRouter.tsx` | Add `case "video"` routing |

### Component Structure

```tsx
// VideoTranscriptionPage.tsx
<div className="h-full flex flex-col">
  <VideoPicker onTranscribe={handleTranscribe} />
  <ProgressIndicator step={progressStep} />
  <TranscriptionViewer segments={segments} />
  <ExportMenu transcriptionId={id} />
</div>
```

### Design System Classes

- `.card` — containers for each section
- `.btn`, `.btn-primary` — action buttons
- `.select` — dropdowns
- `.section`, `.section-title` — section grouping
- Speaker colors: `text-speaker-0` through `text-speaker-5` (custom Tailwind colors)

---

## Phase 7: Frontend — AI Panel (Summary + Chat)

**Goal**: Build AI summary and chat UI integrated into the video transcription page.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/VideoTranscription/AiPanel.tsx` | Tabs for Summary and Chat |
| `src/components/VideoTranscription/AiChatMessage.tsx` | Chat message bubble with role indicator |
| `src/components/Settings/AiSettings.tsx` | AI provider configuration in Settings |

### AI Panel Structure

```tsx
// AiPanel.tsx
<div className="card">
  <div className="tab-bar">
    <button>Summary</button>
    <button>Chat</button>
  </div>
  {activeTab === "summary" && <SummaryView />}
  {activeTab === "chat" && <ChatView />}
</div>
```

### Chat Streaming

- Use Tauri events (`listen("ai-chat-token", ...)`) for streaming
- Backend emits tokens as they arrive from the AI provider
- Frontend appends tokens to the current message in real-time

---

## Phase 8: Settings — AI Configuration

**Goal**: Add AI provider settings panel.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/Settings/AiSettings.tsx` | Provider selector, URL input, model input, test button |

### Files to Modify

| File | Changes |
|------|---------|
| `src/components/Settings/SettingsLayout.tsx` | Add AiSettings to "general" tab |
| `src/hooks/useSettings.ts` | Add `ai` section to `AppConfig` interface |
| `src-tauri/src/settings/config.rs` | Add `AiSettingsConfig` struct with defaults |

### Settings Layout

```
Settings > General > AI Provider
├── Provider: [Ollama ▾] [LM Studio] [DeepSeek]
├── Base URL: [http://localhost:11434/v1]
├── API Key: [***] (only for DeepSeek)
├── Model: [llama3.2]
└── [Test Connection] ✓ Connected
```

---

## Phase 9: Wiring & Integration

**Goal**: Connect all modules, register commands, manage state.

### Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/lib.rs` | Add module declarations, manage states, register all new commands |
| `src-tauri/src/settings/config.rs` | Add `AiSettingsConfig` with defaults |
| `src-tauri/src/history/db.rs` | Add video_transcriptions table creation |
| `src-tauri/Cargo.toml` | Add `speakrs` dependency |

### State Management

```rust
// In lib.rs
struct VideoTranscriptionState {
    db: Arc<Mutex<HistoryDb>>,
    whisper: Arc<Mutex<WhisperEngine>>,
    diarization: Arc<Mutex<DiarizationEngine>>,
}

struct AiState {
    provider: Arc<Mutex<dyn AiProvider>>,
    config: Arc<Mutex<AiConfig>>,
}
```

---

## Phase 10: Testing & Polish

**Goal**: Verify all features, run lint/typecheck, fix issues.

### Verification Checklist

- [ ] `cargo check` passes
- [ ] `cargo test` passes (all existing + new tests)
- [ ] `bun run typecheck` passes
- [ ] `bun run lint` passes
- [ ] Video transcription works end-to-end
- [ ] Export produces valid SRT/VTT/ASS files
- [ ] Speaker diarization labels segments correctly
- [ ] AI summary generates coherent text
- [ ] AI chat streams responses
- [ ] Settings persist across app restarts
- [ ] Error states handled gracefully

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Video Processing): No dependencies — starts immediately
- **Phase 2** (AI Providers): No dependencies — starts immediately
- **Phase 3** (Diarization): No dependencies — starts immediately
- **Phase 4** (Commands): Depends on Phases 1, 2, 3
- **Phase 5** (Export ASS): No dependencies — starts immediately
- **Phase 6** (Frontend Video): Depends on Phase 4
- **Phase 7** (Frontend AI): Depends on Phases 2, 4
- **Phase 8** (Settings): Depends on Phase 2
- **Phase 9** (Wiring): Depends on all previous phases
- **Phase 10** (Testing): Depends on Phase 9

### Parallel Opportunities

- Phases 1, 2, 3, 5 can all run in parallel (no dependencies)
- Phase 8 can run in parallel with Phases 6, 7
- Frontend work (6, 7) can be parallel with Rust backend work (4, 5)

---

## Notes

- FFmpeg must be installed on the system (`pacman -S ffmpeg` on Arch)
- `speakrs` uses `ort` 2.0 (pre-release ONNX Runtime) — acceptable for this use case
- AI streaming uses Tauri events, not WebSocket — simpler and matches existing patterns
- Database migration is additive (new table) — no schema conflicts
- Video cleanup: temp WAV files are deleted after transcription
