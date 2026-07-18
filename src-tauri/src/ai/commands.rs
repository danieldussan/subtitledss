use tauri::{Emitter, State};
use std::sync::{Arc, Mutex};

use super::config::{AiConfig, AiProviderInfo, ChatMessage};
use super::provider::{create_provider};

#[tauri::command]
pub async fn list_ai_providers() -> Result<Vec<AiProviderInfo>, String> {
    Ok(AiProviderInfo::list())
}

#[tauri::command]
pub async fn get_ai_config(state: State<'_, Arc<Mutex<AiConfig>>>) -> Result<AiConfig, String> {
    let config = state.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_ai_config(
    config: AiConfig,
    state: State<'_, Arc<Mutex<AiConfig>>>,
) -> Result<(), String> {
    let mut cfg = state.lock().unwrap();
    *cfg = config;
    Ok(())
}

#[tauri::command]
pub async fn test_ai_connection(config: AiConfig) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let base = config.base_url.trim_end_matches('/').to_string();

    // Try OpenAI-compatible endpoint first: {base}/models
    let openai_url = format!("{}/models", base);
    let mut builder = client.get(&openai_url);
    if let Some(ref api_key) = config.api_key {
        builder = builder.bearer_auth(api_key);
    }

    match builder.send().await {
        Ok(resp) if resp.status().is_success() => {
            return Ok(format!("Connected to {} ({})", base, resp.status()));
        }
        Ok(_) => {}
        Err(_) => {}
    }

    // Try LM Studio native endpoint: {base}/../api/v1/models
    // If base is "http://host:port/v1", native is "http://host:port/api/v1/models"
    let native_url = if base.ends_with("/v1") {
        let prefix = &base[..base.len() - 2];
        format!("{}api/v1/models", prefix)
    } else {
        format!("{}/api/v1/models", base)
    };

    let mut builder = client.get(&native_url);
    if let Some(ref api_key) = config.api_key {
        builder = builder.bearer_auth(api_key);
    }

    match builder.send().await {
        Ok(resp) if resp.status().is_success() => {
            return Ok(format!("Connected to {} ({})", native_url, resp.status()));
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} from {}: {}", status, native_url, body.chars().take(300).collect::<String>()))
        }
        Err(e) => Err(format!("Connection failed: tried {} and {} — {}", openai_url, native_url, e)),
    }
}

#[tauri::command]
pub async fn ai_summarize(
    transcription_text: String,
    custom_prompt: Option<String>,
    language: Option<String>,
    config_state: State<'_, Arc<Mutex<AiConfig>>>,
) -> Result<String, String> {
    let config = {
        let cfg = config_state.lock().unwrap();
        cfg.clone()
    };

    let lang_name = match language.as_deref() {
        Some("es") | Some("Spanish") => "Spanish",
        Some("en") | Some("English") => "English",
        Some("fr") | Some("French") => "French",
        Some("de") | Some("German") => "German",
        Some("pt") | Some("Portuguese") => "Portuguese",
        Some("it") | Some("Italian") => "Italian",
        Some("ja") | Some("Japanese") => "Japanese",
        Some("zh") | Some("Chinese") => "Chinese",
        Some("ko") | Some("Korean") => "Korean",
        Some("ru") | Some("Russian") => "Russian",
        Some("ar") | Some("Arabic") => "Arabic",
        _ => "the same language as the transcription",
    };

    let system_prompt = custom_prompt.unwrap_or_else(|| {
        format!(
            "You are a helpful assistant that summarizes video transcriptions. \
             Provide a concise summary covering the main topics, key points, and any \
             conclusions or action items mentioned. Use bullet points for clarity. \
             IMPORTANT: Always respond in {}.",
            lang_name
        )
    });

    let messages = vec![ChatMessage::user(&format!(
        "Please summarize the following transcription:\n\n{}",
        transcription_text
    ))];

    let provider = create_provider(config);
    provider
        .chat(&system_prompt, &messages)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_chat(
    _message: String,
    system_context: String,
    history: Vec<ChatMessage>,
    language: Option<String>,
    config_state: State<'_, Arc<Mutex<AiConfig>>>,
) -> Result<String, String> {
    let config = {
        let cfg = config_state.lock().unwrap();
        cfg.clone()
    };

    let lang_instruction = match language.as_deref() {
        Some("es") | Some("Spanish") => "Always respond in Spanish.",
        Some("en") | Some("English") => "Always respond in English.",
        Some("fr") | Some("French") => "Always respond in French.",
        Some("de") | Some("German") => "Always respond in German.",
        Some("pt") | Some("Portuguese") => "Always respond in Portuguese.",
        Some("it") | Some("Italian") => "Always respond in Italian.",
        Some("ja") | Some("Japanese") => "Always respond in Japanese.",
        Some("zh") | Some("Chinese") => "Always respond in Chinese.",
        Some("ko") | Some("Korean") => "Always respond in Korean.",
        Some("ru") | Some("Russian") => "Always respond in Russian.",
        Some("ar") | Some("Arabic") => "Always respond in Arabic.",
        _ => "Always respond in the same language as the user's message.",
    };

    let system_prompt = format!(
        "You are a helpful assistant discussing a video transcription. \
         Use the following transcription as context for answering questions:\n\n\
         --- TRANSCRIPTION ---\n{}\n--- END ---\n\n\
         Answer questions about the content, clarify points, and provide insights.\n\n{}",
        system_context, lang_instruction
    );

    let provider = create_provider(config);
    provider
        .chat(&system_prompt, &history)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_chat_stream_start(
    _message: String,
    system_context: String,
    history: Vec<ChatMessage>,
    language: Option<String>,
    config_state: State<'_, Arc<Mutex<AiConfig>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let config = {
        let cfg = config_state.lock().unwrap();
        cfg.clone()
    };

    let lang_instruction = match language.as_deref() {
        Some("es") | Some("Spanish") => "Always respond in Spanish.",
        Some("en") | Some("English") => "Always respond in English.",
        Some("fr") | Some("French") => "Always respond in French.",
        Some("de") | Some("German") => "Always respond in German.",
        Some("pt") | Some("Portuguese") => "Always respond in Portuguese.",
        Some("it") | Some("Italian") => "Always respond in Italian.",
        Some("ja") | Some("Japanese") => "Always respond in Japanese.",
        Some("zh") | Some("Chinese") => "Always respond in Chinese.",
        Some("ko") | Some("Korean") => "Always respond in Korean.",
        Some("ru") | Some("Russian") => "Always respond in Russian.",
        Some("ar") | Some("Arabic") => "Always respond in Arabic.",
        _ => "Always respond in the same language as the user's message.",
    };

    let system_prompt = format!(
        "You are a helpful assistant discussing a video transcription. \
         Use the following transcription as context for answering questions:\n\n\
         --- TRANSCRIPTION ---\n{}\n--- END ---\n\n\
         Answer questions about the content, clarify points, and provide insights.\n\n{}",
        system_context, lang_instruction
    );

    let provider = create_provider(config);
    let mut rx = provider
        .chat_stream(&system_prompt, &history)
        .await
        .map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        while let Some(token) = rx.recv().await {
            let _ = app_handle.emit("ai-chat-token", serde_json::json!({
                "token": token,
            }));
        }
        let _ = app_handle.emit("ai-chat-done", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn ai_translate_text(
    text: String,
    target_language: String,
    config_state: State<'_, Arc<Mutex<AiConfig>>>,
) -> Result<String, String> {
    let config = {
        let cfg = config_state.lock().unwrap();
        cfg.clone()
    };

    let lang_name = match target_language.as_str() {
        "es" | "Spanish" => "Spanish",
        "en" | "English" => "English",
        "fr" | "French" => "French",
        "de" | "German" => "German",
        "pt" | "Portuguese" => "Portuguese",
        "it" | "Italian" => "Italian",
        "ja" | "Japanese" => "Japanese",
        "zh" | "Chinese" => "Chinese",
        "ko" | "Korean" => "Korean",
        "ru" | "Russian" => "Russian",
        "ar" | "Arabic" => "Arabic",
        _ => &target_language,
    };

    let system_prompt = format!(
        "You are a professional translator. Translate the following text to {}. \
         Preserve the original meaning, tone, and formatting. \
         Only output the translation, nothing else.",
        lang_name
    );

    let messages = vec![ChatMessage::user(&text)];

    let provider = create_provider(config);
    provider
        .chat(&system_prompt, &messages)
        .await
        .map_err(|e| e.to_string())
}
