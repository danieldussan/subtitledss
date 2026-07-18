pub mod audio;
pub mod commands;
pub mod history;
pub mod models;
pub mod overlay;
pub mod pipeline;
pub mod settings;
pub mod translation;
pub mod vad;
pub mod whisper;
pub mod video;
pub mod ai;
pub mod diarization;

use std::sync::{Arc, Mutex, atomic::{AtomicU32, AtomicUsize}};
use tauri::Manager;
use tauri::Emitter;
use tauri::Listener;
use settings::AppConfig;
use whisper::WhisperEngine;
use history::HistoryDb;
use overlay::{OverlayManager, OverlayConfig};
use audio::{AudioCapture, RingBuffer};
use whisper::model::ModelManager;
use pipeline::TranscriptionPipeline;
use translation::marian::MarianEngine;
use commands::video_transcription::VideoTranscriptionState;
use diarization::engine::DiarizationEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = AppConfig::load();
    let whisper_engine = Arc::new(Mutex::new(WhisperEngine::new()));
    let overlay_config = OverlayConfig {
        x: config.overlay.x,
        y: config.overlay.y,
        width: config.overlay.width,
        height: config.overlay.height,
        opacity: config.overlay.opacity,
        always_on_top: config.overlay.always_on_top,
        click_through: config.overlay.click_through,
        font_size: config.overlay.font_size,
        font_color: config.overlay.font_color.clone(),
        background_color: config.overlay.background_color.clone(),
        auto_hide: config.overlay.auto_hide,
        auto_hide_delay: config.overlay.auto_hide_delay,
        display_duration_ms: config.overlay.display_duration_ms,
        fade_duration_ms: config.overlay.fade_duration_ms,
        max_visible_lines: config.overlay.max_visible_lines,
        line_gap: config.overlay.line_gap,
        max_line_width: config.overlay.max_line_width,
    };
    let overlay_manager = Arc::new(Mutex::new(OverlayManager::new(overlay_config)));

    let db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("subtitledss")
        .join("history.db");

    let history_db = match HistoryDb::new(&db_path) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => {
            tracing::error!("Failed to initialize history database: {}", e);
            std::process::exit(1);
        }
    };

    let models_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("subtitledss")
        .join("models");

    let model_manager = Arc::new(Mutex::new(ModelManager::new(models_dir.clone())));

    let audio_capture = Arc::new(Mutex::new(AudioCapture::new()));
    let audio_buffer = Arc::new(Mutex::new(RingBuffer::default()));
    let pipeline = Arc::new(Mutex::new(TranscriptionPipeline::new()));
    let actual_sample_rate = Arc::new(AtomicU32::new(16000));
    let actual_channels = Arc::new(AtomicUsize::new(1));

    let marian_engine = Arc::new(Mutex::new(MarianEngine::new(models_dir.clone())));

    // Initialize video transcription state
    let diarization_engine = Arc::new(Mutex::new(DiarizationEngine::new()));

    let video_transcription_state = VideoTranscriptionState {
        db: history_db.clone(),
        whisper: whisper_engine.clone(),
        diarization: diarization_engine,
    };

    // Initialize AI config — convert AiSettingsConfig (String provider) to AiConfig (enum provider)
    let ai_provider_type = match config.ai.provider.to_lowercase().as_str() {
        "lmstudio" | "lm_studio" => ai::config::AiProviderType::LmStudio,
        "deepseek" => ai::config::AiProviderType::DeepSeek,
        _ => ai::config::AiProviderType::Ollama,
    };
    let ai_config = Arc::new(Mutex::new(ai::config::AiConfig {
        provider: ai_provider_type,
        base_url: config.ai.base_url.clone(),
        api_key: config.ai.api_key.clone(),
        model: config.ai.model.clone(),
    }));

    let config_arc = Arc::new(Mutex::new(config));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(whisper_engine)
        .manage(overlay_manager)
        .manage(history_db)
        .manage(config_arc)
        .manage(model_manager)
        .manage(marian_engine)
        .manage(audio_capture)
        .manage(audio_buffer)
        .manage(pipeline)
        .manage(actual_sample_rate)
        .manage(actual_channels)
        .manage(video_transcription_state)
        .manage(ai_config)
        .invoke_handler(tauri::generate_handler![
            commands::transcription::transcribe_audio,
            commands::settings::get_config,
            commands::settings::save_config,
            commands::settings::list_audio_devices,
            commands::history::get_history,
            commands::history::search_history,
            commands::history::clear_history,
            commands::capture::start_capture,
            commands::capture::stop_capture,
            commands::capture::get_audio_level,
            commands::models::download_model,
            commands::models::delete_model,
            commands::models::list_downloaded_models,
            commands::models::load_model,
            commands::export::export_history,
            commands::overlay::toggle_overlay,
            commands::overlay::show_overlay,
            commands::overlay::hide_overlay,
            commands::translation::download_marian_model,
            commands::translation::check_marian_model,
            commands::translation::delete_marian_model,
            commands::translation::list_marian_models,
            commands::video_transcription::transcribe_video,
            commands::video_transcription::list_video_transcriptions,
            commands::video_transcription::delete_video_transcription,
            commands::video_transcription::get_video_transcription,
            commands::video_transcription::update_video_transcription_summary,
            commands::video_transcription::export_video_transcription,
            ai::commands::list_ai_providers,
            ai::commands::get_ai_config,
            ai::commands::save_ai_config,
            ai::commands::test_ai_connection,
            ai::commands::ai_summarize,
            ai::commands::ai_chat,
            ai::commands::ai_chat_stream_start,
            ai::commands::ai_translate_text,
        ])
        .setup(|app| {
            // Auto-load whisper model if configured
            let config = {
                let cfg = app.state::<Arc<Mutex<AppConfig>>>();
                let cfg = cfg.lock().unwrap();
                cfg.clone()
            };

            let model_name = &config.whisper.model;
            let models_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("subtitledss")
                .join("models");
            let model_path = models_dir.join(format!("ggml-{}.bin", model_name));

            if model_path.exists() {
                let whisper = app.state::<Arc<Mutex<WhisperEngine>>>();
                let mut engine = whisper.lock().unwrap();
                match engine.load_model(&model_path) {
                    Ok(()) => tracing::info!("Auto-loaded Whisper model: {}", model_name),
                    Err(e) => tracing::error!("Failed to load model {}: {}", model_name, e),
                }
            } else {
                tracing::info!("Model '{}' not found at {:?}, skipping auto-load", model_name, model_path);
            }

            // Initialize video transcriptions table
            {
                let db = app.state::<Arc<Mutex<HistoryDb>>>();
                let db = db.lock().unwrap();
                if let Err(e) = db.init_video_transcriptions_table() {
                    tracing::error!("Failed to init video transcriptions table: {}", e);
                }
            }

            // Show overlay window
            if let Some(overlay_window) = app.get_webview_window("overlay") {
                let _ = overlay_window.show();
            }

            // System tray
            use tauri::tray::{TrayIconBuilder, TrayIconEvent};
            use tauri::menu::{Menu, MenuItem};

            let start_stop = MenuItem::with_id(app, "start_stop", "Start Capture", true, None::<&str>)?;
            let show_overlay = MenuItem::with_id(app, "show_overlay", "Show Overlay", true, None::<&str>)?;
            let show_window = MenuItem::with_id(app, "show_window", "Show Window", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&start_stop, &show_overlay, &show_window, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("subtitledss — Idle")
                .menu(&menu)
                .on_menu_event(move |app: &tauri::AppHandle, event: tauri::menu::MenuEvent| {
                    let id = event.id().as_ref();
                    match id {
                        "start_stop" => {
                            tracing::info!("Tray: toggle capture");
                            let _ = app.emit("toggle-capture", ());
                        }
                        "show_overlay" => {
                            tracing::info!("Tray: toggle overlay");
                            let _ = app.emit("toggle-overlay", ());
                        }
                        "show_window" => {
                            if let Some(main) = app.get_webview_window("main") {
                                let _ = main.show();
                                let _ = main.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event: tauri::tray::TrayIconEvent| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(main) = app.get_webview_window("main") {
                            let _ = main.show();
                            let _ = main.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Sync tray menu with app state
            let app_handle_for_sync = app.handle().clone();
            app.listen("capture-state-changed", move |event| {
                let capturing = event.payload()
                    .trim_matches('"')
                    == "true" || event.payload().contains("\"capturing\":true");

                let label = if capturing { "Stop Capture" } else { "Start Capture" };
                let tooltip = if capturing { "subtitledss — Capturing" } else { "subtitledss — Idle" };

                if let Some(tray) = app_handle_for_sync.tray_by_id("main-tray") {
                    if let Ok(new_menu) = Menu::with_items(&app_handle_for_sync, &[
                        &MenuItem::with_id(&app_handle_for_sync, "start_stop", label, true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_sync, "show_overlay", "Show Overlay", true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_sync, "show_window", "Show Window", true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_sync, "quit", "Quit", true, None::<&str>).unwrap(),
                    ]) {
                        let _ = tray.set_menu(Some(new_menu));
                    }
                    let _ = tray.set_tooltip(Some(tooltip));
                }
            });

            let app_handle_for_overlay = app.handle().clone();
            app.listen("overlay-state-changed", move |event| {
                let visible = event.payload()
                    .trim_matches('"')
                    == "true" || event.payload().contains("\"visible\":true");

                let label = if visible { "Hide Overlay" } else { "Show Overlay" };

                if let Some(tray) = app_handle_for_overlay.tray_by_id("main-tray") {
                    if let Ok(new_menu) = Menu::with_items(&app_handle_for_overlay, &[
                        &MenuItem::with_id(&app_handle_for_overlay, "start_stop", "Start Capture", true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_overlay, "show_overlay", label, true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_overlay, "show_window", "Show Window", true, None::<&str>).unwrap(),
                        &MenuItem::with_id(&app_handle_for_overlay, "quit", "Quit", true, None::<&str>).unwrap(),
                    ]) {
                        let _ = tray.set_menu(Some(new_menu));
                    }
                }
            });

            // Global shortcuts
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

            let ctrl_shift_s = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
            app.global_shortcut().on_shortcut(ctrl_shift_s, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::info!("Global shortcut: toggle capture");
                    let _ = app.emit("toggle-capture", ());
                }
            })?;

            let ctrl_shift_o = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyO);
            app.global_shortcut().on_shortcut(ctrl_shift_o, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::info!("Global shortcut: toggle overlay");
                    let _ = app.emit("toggle-overlay", ());
                }
            })?;

            let ctrl_shift_t = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT);
            app.global_shortcut().on_shortcut(ctrl_shift_t, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    tracing::info!("Global shortcut: toggle translation");
                    let _ = app.emit("toggle-translation", ());
                }
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
