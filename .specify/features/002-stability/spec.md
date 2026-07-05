# Feature: Phase 1 — Stability

## Goal
Make the project production-ready for public GitHub release with proper testing, CI/CD, and error handling.

## User Stories

### US-1: Unit Tests for Core Modules
**As a** developer
**I want** automated tests for critical Rust modules
**So that** regressions are caught before they reach users

**Acceptance Criteria:**
- [ ] `audio/buffer.rs`: push, drain, peek, capacity, overflow tests
- [ ] `vad/detector.rs`: silence detection, voice detection, threshold behavior
- [ ] `settings/config.rs`: load defaults, save/load roundtrip, missing file handling
- [ ] `history/db.rs`: insert, get_all, search, clear, FTS5 match
- [ ] `whisper/engine.rs`: new engine, load model (mock), is_loaded states
- [ ] `pipeline/transcriber.rs`: chunk collection logic, silence timeout
- [ ] All tests pass with `cargo test`
- [ ] Minimum 80% line coverage on critical modules

### US-2: CI/CD Pipeline
**As a** contributor
**I want** automated checks on every push/PR
**So that** code quality is maintained

**Acceptance Criteria:**
- [ ] `.github/workflows/ci.yml` created
- [ ] Runs on push to main and PRs
- [ ] Steps: checkout → install Rust/Bun → cargo check → cargo test → bun install → bun run typecheck → bun run build
- [ ] Caches Rust build artifacts and Bun node_modules
- [ ] Fails fast on any step failure
- [ ] Badge added to README

### US-3: Error Handling in UI
**As a** user
**I want** to see clear error messages when something goes wrong
**So that** I can understand and fix the issue

**Acceptance Criteria:**
- [ ] Toast notification system (bottom-right, auto-dismiss 5s)
- [ ] Errors from `invoke()` calls shown as toasts
- [ ] Specific messages: "Model not loaded", "Audio device not found", "Download failed", "Whisper error"
- [ ] Warning toasts for non-critical issues
- [ ] Success toasts for actions: "Model loaded", "Settings saved", "Capture started"
- [ ] Toast stack max 3 visible, oldest dismissed first

### US-4: Project Documentation
**As a** contributor
**I want** clear project conventions documented
**So that** I can contribute without asking questions

**Acceptance Criteria:**
- [ ] `AGENTS.md` with: tech stack, build commands, lint rules, commit conventions
- [ ] `README.md` with: description, screenshots, install instructions, usage, contributing
- [ ] `.github/CONTRIBUTING.md` with: setup, PR process, code style
- [ ] `LICENSE` confirmed as MIT

## Technical Notes

### Test Framework
- Rust: built-in `#[cfg(test)]` with `cargo test`
- No external test framework needed for v1
- Mock audio data for whisper tests (generate sine waves)

### Toast System
- Create `src/components/ui/Toast.tsx` with framer-motion animations
- Create `src/hooks/useToast.ts` for global toast state
- Toast types: `success`, `error`, `warning`, `info`
- Stack from bottom-right, slide in/out

### CI Cache Keys
```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

## File Structure
```
.github/
├── workflows/
│   └── ci.yml
├── CONTRIBUTING.md
└── prompts/
    └── (existing speckit prompts)

src/
├── components/
│   └── ui/
│       └── Toast.tsx
├── hooks/
│   └── useToast.ts
└── (existing files)

AGENTS.md
README.md
LICENSE
```

## Testing Checklist
- [ ] `cargo test` — all pass
- [ ] `cargo clippy` — no warnings
- [ ] `bun run typecheck` — no errors
- [ ] `bun run build` — succeeds
- [ ] CI pipeline green on first push
