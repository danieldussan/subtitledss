# Feature Specification: Video Transcription, AI Chat & Speaker Diarization

**Feature Branch**: `009-video-transcription-ai`

**Created**: 2026-07-17

**Status**: Draft

**Input**: User description: "Add a video section to transcribe local video files, export subtitles in SRT/VTT/TXT/JSON/ASS formats, generate AI summaries, chat with the transcription using local/remote LLMs (Ollama, LM Studio, DeepSeek), and detect different speakers."

## User Scenarios & Testing

### User Story 1 - Transcribe Local Video File (Priority: P1)

As a user, I want to select a local video file and get a complete transcription with timestamps, so I can create subtitles from any video on my computer.

**Why this priority**: This is the core feature. Without video-to-text transcription, no other video feature works. This delivers immediate value — the user gets a full transcription they can export.

**Independent Test**: Can be fully tested by selecting an MP4 file, clicking Transcribe, and verifying the transcription appears with correct timestamps and text. Delivers a working video transcription pipeline.

**Acceptance Scenarios**:

1. **Given** the user navigates to the Video section, **When** the page loads, **Then** a file picker button is visible with language selector.
2. **Given** the user clicks "Choose File", **When** a video file is selected (mp4, mkv, avi, webm, mov), **Then** the file metadata (name, duration, format) is displayed.
3. **Given** a video file is selected, **When** the user clicks "Transcribe", **Then** a progress indicator shows extraction and transcription status.
4. **Given** transcription is in progress, **When** the user views the page, **Then** progress updates are shown (extracting audio, transcribing, processing).
5. **Given** transcription completes, **When** the result is displayed, **Then** segments show start/end timestamps and transcribed text.
6. **Given** transcription completes, **When** the user views the result, **Then** the full text is also available as a combined view.

---

### User Story 2 - Export Transcription as Subtitle Files (Priority: P1)

As a user with a completed video transcription, I want to export it in standard subtitle formats so I can use it in video players and editors.

**Why this priority**: Export is the primary use case for video transcription. Users transcribe videos specifically to get subtitle files. Must be available immediately after transcription.

**Independent Test**: Can be tested by completing a transcription and clicking each export format button, verifying the downloaded file has correct content and formatting.

**Acceptance Scenarios**:

1. **Given** a completed transcription, **When** the user clicks Export, **Then** format options are shown: SRT, VTT, TXT, JSON, ASS/SSA.
2. **Given** the user selects SRT format, **When** the file is exported, **Then** the SRT file contains numbered blocks with `HH:MM:SS,mmm --> HH:MM:SS,mmm` timestamps and text.
3. **Given** the user selects VTT format, **When** the file is exported, **Then** the VTT file starts with `WEBVTT` header and uses `HH:MM:SS.mmm` timestamps.
4. **Given** the user selects ASS format, **When** the file is exported, **Then** the ASS file contains Script Info, V4+ Styles, and Dialogue sections with proper formatting.
5. **Given** the user selects JSON format, **When** the file is exported, **Then** the JSON contains an array of segment objects with start, end, text, and speaker fields.
6. **Given** the user selects TXT format, **When** the file is exported, **Then** the file contains plain text with timestamps as headers.

---

### User Story 3 - Speaker Diarization (Priority: P2)

As a user transcribing a video with multiple speakers, I want the transcription to identify who spoke when, so I can attribute dialogue to specific people.

**Why this priority**: Speaker diarization significantly enhances transcription quality for interviews, podcasts, meetings, and conversations. It's a differentiating feature.

**Independent Test**: Can be tested by transcribing a video with 2+ speakers and verifying the transcription labels segments with speaker identifiers (Speaker_0, Speaker_1, etc.).

**Acceptance Scenarios**:

1. **Given** the user enables diarization, **When** transcription completes, **Then** each segment includes a speaker label (e.g., "Speaker_0", "Speaker_1").
2. **Given** diarization is enabled, **When** the transcription is displayed, **Then** different speakers are visually distinguished (color coding or icons).
3. **Given** diarization is enabled, **When** the transcription is exported, **Then** speaker labels are included in the exported file (ASS format supports character names).
4. **Given** a single-speaker video, **When** diarization runs, **Then** all segments are assigned to "Speaker_0" without hallucinating additional speakers.
5. **Given** diarization is disabled, **When** transcription runs, **Then** no speaker labels are shown (segments only have text and timestamps).

---

### User Story 4 - AI Summary Generation (Priority: P2)

As a user with a completed transcription, I want to generate an AI-powered summary of the video content, so I can quickly understand the key points without reading the full text.

**Why this priority**: Summary provides immediate value for long videos. It's a natural first step before chatting with the content.

**Independent Test**: Can be tested by completing a transcription, clicking "Generate Summary", and verifying a coherent summary is produced using the configured AI provider.

**Acceptance Scenarios**:

1. **Given** a completed transcription, **When** the user clicks "Generate Summary", **Then** the AI provider generates a concise summary of the video content.
2. **Given** the summary is generated, **When** the user views the result, **Then** the summary is displayed in a dedicated panel with the option to regenerate.
3. **Given** the AI provider is not configured, **When** the user clicks "Generate Summary", **Then** a prompt to configure the AI provider in Settings is shown.
4. **Given** the summary is generated, **When** the user navigates away and returns, **Then** the summary persists (stored in the database).

---

### User Story 5 - AI Chat with Transcription (Priority: P2)

As a user with a completed transcription, I want to have a conversation with an AI about the video content, so I can ask questions, get clarifications, and explore topics discussed.

**Why this priority**: Chat enables interactive exploration of video content. It's the most engaging AI feature and differentiates from simple transcription tools.

**Independent Test**: Can be tested by starting a chat session, sending messages, and verifying the AI responds contextually about the transcription content.

**Acceptance Scenarios**:

1. **Given** a completed transcription, **When** the user opens the AI Chat panel, **Then** a chat interface with message input and history is displayed.
2. **Given** the user sends a message, **When** the AI processes it, **Then** the response is streamed token-by-token to the chat.
3. **Given** the chat history exists, **When** the user sends follow-up messages, **Then** the AI maintains conversation context.
4. **Given** the user changes the AI provider, **When** a new message is sent, **Then** the new provider handles the request.
5. **Given** the AI provider is offline, **When** the user sends a message, **Then** an error message is shown with retry option.

---

### User Story 6 - AI Provider Configuration (Priority: P2)

As a user, I want to configure which AI provider to use (Ollama, LM Studio, or DeepSeek) with custom URLs and models, so I can use my preferred LLM setup.

**Why this priority**: Provider configuration is required for all AI features. Without it, summary and chat don't work.

**Independent Test**: Can be tested by opening AI settings, selecting a provider, entering URL/model, and verifying the connection works.

**Acceptance Scenarios**:

1. **Given** the user opens AI Settings, **When** the settings load, **Then** provider options are shown: Ollama, LM Studio, DeepSeek.
2. **Given** the user selects a provider, **When** the default values load, **Then** base URL and model are pre-filled with provider defaults.
3. **Given** the user enters custom URL and model, **When** they click "Test Connection", **Then** a success or failure indicator is shown.
4. **Given** the user saves AI settings, **When** the settings are persisted, **Then** they are used for all subsequent AI operations.

---

### Edge Cases

- What happens when the video file is corrupt or unsupported? → Show error toast with details.
- What happens when FFmpeg is not installed? → Show install instructions in the error message.
- What happens when the video is very long (>2 hours)? → Show progress and allow cancellation.
- What happens when the AI provider connection drops mid-chat? → Show error, allow retry.
- What happens when diarization takes too long? → Show progress, allow cancellation.
- What happens when the user selects an audio-only file? → Treat as video with 0 duration, transcribe directly.

## Requirements

### Functional Requirements

- **FR-001**: System MUST allow users to select local video files (mp4, mkv, avi, webm, mov, mp3, wav, flac).
- **FR-002**: System MUST extract audio from video files using FFmpeg (16kHz mono WAV).
- **FR-003**: System MUST transcribe extracted audio using the existing Whisper engine.
- **FR-004**: System MUST display transcription with timestamps and optional speaker labels.
- **FR-005**: System MUST export transcriptions in SRT, VTT, TXT, JSON, and ASS/SSA formats.
- **FR-006**: System MUST detect speakers using diarization when enabled.
- **FR-007**: System MUST generate AI summaries of transcriptions using configured provider.
- **FR-008**: System MUST support chat conversations about transcription content.
- **FR-009**: System MUST support Ollama, LM Studio, and DeepSeek as AI providers.
- **FR-010**: System MUST persist video transcriptions and summaries in SQLite.
- **FR-011**: System MUST stream AI chat responses token-by-token.
- **FR-012**: System MUST show progress indicators for long-running operations.
- **FR-013**: System MUST handle FFmpeg not being installed with a clear error message.

### Key Entities

- **VideoTranscription**: Represents a completed video transcription. Contains video path, name, duration, language, full text, summary, segments (JSON), and creation timestamp.
- **DiarizedSegment**: A transcription segment with start time, end time, text, and optional speaker label.
- **AiProviderConfig**: Configuration for an AI provider (type, base URL, API key, model).
- **ChatMessage**: A message in an AI chat session (role, content, timestamp).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can transcribe a 10-minute video in under 2 minutes on a modern CPU.
- **SC-002**: Export produces valid SRT/VTT files playable in VLC/MPV.
- **SC-003**: Speaker diarization correctly identifies 2+ speakers with >80% accuracy on clear audio.
- **SC-004**: AI summary generation completes in under 10 seconds for a 10-minute transcription.
- **SC-005**: AI chat responses stream with <500ms first-token latency for local providers.
- **SC-006**: All export formats pass format-specific validators.

## Assumptions

- FFmpeg is installed on the system (standard on Arch Linux, trivial to install).
- AI providers (Ollama, LM Studio) are running locally or accessible via network.
- The existing Whisper engine and model system are reused for video transcription.
- Speaker diarization models (~30MB) are downloaded on first use.
- DeepSeek requires an API key; Ollama and LM Studio are local-only.
- Video files are local (no URL/download support in this version).
