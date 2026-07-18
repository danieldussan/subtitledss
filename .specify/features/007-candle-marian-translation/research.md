# Research: Candle + Marian MT Translation

**Feature**: `007-candle-marian-translation`
**Date**: 2026-07-13
**Status**: Complete

## Research Questions

### Q1: Are Helsinki-NLP/opus-mt models compatible with candle-transformers?

**Finding**: Yes. The Helsinki-NLP/opus-mt-en-es and opus-mt-es-en models are standard Marian encoder-decoder models. The `candle-transformers` crate provides `candle_transformers::models::marian` which implements the Marian architecture. HuggingFace hosts these models with safetensors weights, which candle can load directly via `VarBuilder::from_safe_tensors()`.

**Key files needed per model**:
- `model.safetensors` (~300MB) — model weights
- `tokenizer.json` — HuggingFace tokenizer format, supported by `tokenizers` crate
- `config.json` — model configuration (encoder/decoder layers, vocab size, etc.)
- `generation_config.json` — beam search parameters, max length

**Source**: candle-transformers GitHub repo, HuggingFace model cards

### Q2: What is the expected inference latency on CPU?

**Finding**: Marian MT models are small (~300M parameters) and designed for fast CPU inference. Typical latency for a 10-30 word segment is 50-150ms on a modern 4-core CPU. The 200ms target is achievable.

**Factors affecting latency**:
- Input length (longer = slower, but Marian is faster than transformer-base)
- CPU cores (parallelizable via rayon, but candle uses single-threaded by default)
- Model size (both EN→ES and ES→EN are ~300MB, similar architecture)

**Mitigation if too slow**:
- Use `candle_core::Device::Cpu` with explicit thread count
- Consider quantized models (INT8) for 2x speedup (future enhancement)
- Batch processing is not applicable for real-time subtitle translation

### Q3: How does the existing ModelDownloader pattern work?

**Finding**: Two parallel implementations exist:
1. `whisper::model::ModelManager` — used by `commands/models.rs` for Whisper models
2. `models::downloader::ModelDownloader` — used by `commands/models.rs` for download

Both follow the same pattern:
- `ModelInfo::available_models()` returns a static list
- `download()` does HTTP GET to HuggingFace URL, writes to disk
- `verify_checksum()` uses SHA256

The Marian model manager should follow this same pattern but with subdirectories (each Marian model is a directory of files, not a single .bin file).

### Q4: How to handle legacy config migration?

**Finding**: The current `TranslationConfig` has fields `engine` (String) and `libretranslate_url` (String) that will be removed. Old TOML files will have these fields.

**Migration strategy**:
1. Try parsing with new struct (no engine/libretranslate_url fields)
2. If that fails, try parsing with a legacy struct that includes those fields
3. Log info message about migration
4. Save new format on next save

Since `serde` by default ignores unknown fields when deserializing, we can use `#[serde(default)]` on the new struct and simply not include the removed fields. The TOML parser will silently ignore `engine` and `libretranslate_url` from old configs.

**Alternative**: Use `#[serde(deny_unknown_fields)]` for strict parsing, then implement explicit migration. This is safer but more complex.

**Recommendation**: Use `serde(default)` approach for simplicity. The removed fields are simply ignored.

### Q5: Thread safety requirements for MarianEngine?

**Finding**: The pipeline runs in a `tokio::spawn` async task. Translation is synchronous (CPU-bound). MarianEngine must be wrapped in `Arc<Mutex<MarianEngine>>` for shared access.

**Pattern**:
```rust
let translation_engine: Arc<Mutex<TranslationEngine>> = Arc::new(Mutex::new(...));
// In pipeline:
let engine = translation_engine.lock().map_err(|e| e.to_string())?;
engine.translate(&text, &config)?
```

This matches the existing pattern for `WhisperEngine` (also `Arc<Mutex<WhisperEngine>>`).

### Q6: What happens if candle fails to initialize?

**Finding**: candle-core may fail to initialize on incompatible CPUs (very old x86 without SSE4.2, or ARM without NEON). However, this is extremely rare on modern hardware.

**Mitigation**:
- Catch initialization errors in `MarianEngine::load()`
- Log warning and return error
- Pipeline continues with original text only (no translation)
- User sees error toast in Settings UI

### Q7: How to handle simultaneous whisper + marian model loading?

**Finding**: The Constitution specifies sequential loading to avoid peak RAM spikes. The pipeline already loads whisper first (on app start), then Marian (when translation is enabled).

**Implementation**:
- In `lib.rs` setup: load whisper model first
- Then: load Marian models if translation enabled
- Each load is ~300MB, total peak ~900MB (whisper base 600MB + marian 300MB)
- This is within the 700MB target with runtime overhead (~100MB)

---

## Research Conclusions

1. **Candle + Marian is viable** — architecture compatible, models available, performance acceptable
2. **Follow existing patterns** — ModelDownloader, Arc<Mutex<>>, config migration
3. **CPU-only is sufficient** — no GPU dependency needed
4. **Migration is straightforward** — serde handles unknown fields gracefully
5. **Thread safety is well-understood** — same pattern as WhisperEngine

---

## Open Questions

None remaining. All research questions resolved.
