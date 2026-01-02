mod core;
mod error;
mod i18n;
mod security;

use crate::core::fast_scaler::FastScaler;
use crate::core::config::{Config, HardwareEncoder, QualityPreset};
use crate::i18n::{Language, get_all_translations, save_language_preference};
use crate::security::{validate_file_path, validate_output_path, validate_deletion_path, validate_video_file};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Once;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};

// Initialize FFmpeg once at startup
static INIT_FFMPEG: Once = Once::new();

fn ensure_ffmpeg() -> std::result::Result<(), String> {
    use ffmpeg_sidecar::download::auto_download;

    INIT_FFMPEG.call_once(|| {
        println!("Initializing FFmpeg...");
        // This will download FFmpeg if it's not already available
        if let Err(e) = auto_download() {
            eprintln!("Failed to download FFmpeg: {}", e);
        } else {
            println!("FFmpeg initialized successfully");
        }
    });
    Ok(())
}

/// Tauri command to process a video file
#[tauri::command]
async fn process_video(
    app: AppHandle,
    input_path: String,
    output_path: String,
    scale: Option<u32>,
    quality: Option<String>,
    hardware_encoder: Option<String>,
) -> std::result::Result<String, String> {
    // Ensure FFmpeg is initialized
    ensure_ffmpeg()?;

    // SECURITY: Validate input path
    let input = PathBuf::from(&input_path);
    let validated_input = validate_video_file(&input)
        .map_err(|e| format!("Input validation failed: {}", e))?;

    // SECURITY: Validate output path (must be in user-accessible directory)
    let output = PathBuf::from(&output_path);
    let validated_output = validate_output_path(&output)
        .map_err(|e| format!("Output validation failed: {}", e))?;

    // Create config
    let mut config = Config::new();

    if let Some(s) = scale {
        config = config.with_scale(s);
    }

    if let Some(q) = quality {
        let quality_preset = match q.as_str() {
            "ultra_fast" => QualityPreset::UltraFast,
            "balanced" => QualityPreset::Balanced,
            "high_quality" => QualityPreset::HighQuality,
            _ => QualityPreset::Balanced,
        };
        config = config.with_quality_preset(quality_preset);
    }

    if let Some(he) = hardware_encoder {
        let encoder = match he.as_str() {
            "amd" => HardwareEncoder::AMD,
            "nvidia" => HardwareEncoder::NVIDIA,
            "intel" => HardwareEncoder::Intel,
            _ => HardwareEncoder::None,
        };
        config = config.with_hardware_encoder(encoder);
    }

    // Create progress callback that emits Tauri events
    let app_handle = Arc::new(app);
    let progress_callback = Some(Box::new(move |current: usize, total: usize, percentage: f32| {
        let _ = app_handle.emit("video-progress", serde_json::json!({
            "current": current,
            "total": total,
            "percentage": percentage
        }));
    }) as crate::core::fast_scaler::ProgressCallback);

    // Process video
    match FastScaler::process_video(&validated_input, &validated_output, &config, progress_callback) {
        Ok(_) => Ok(output_path),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri command to generate a 10-second sample
#[tauri::command]
async fn generate_sample(
    app: AppHandle,
    input_path: String,
    output_path: String,
) -> std::result::Result<String, String> {
    // Ensure FFmpeg is initialized
    ensure_ffmpeg()?;

    // SECURITY: Validate input path
    let input = PathBuf::from(&input_path);
    let validated_input = validate_video_file(&input)
        .map_err(|e| format!("Input validation failed: {}", e))?;

    // SECURITY: Validate output path (must be in user-accessible directory)
    let output = PathBuf::from(&output_path);
    let validated_output = validate_output_path(&output)
        .map_err(|e| format!("Output validation failed: {}", e))?;

    let config = Config::new()
        .with_scale(4)
        .with_preview_duration(10); // 10 second sample

    // Create progress callback that emits Tauri events
    let app_handle = Arc::new(app);
    let progress_callback = Some(Box::new(move |current: usize, total: usize, percentage: f32| {
        let _ = app_handle.emit("video-progress", serde_json::json!({
            "current": current,
            "total": total,
            "percentage": percentage
        }));
    }) as crate::core::fast_scaler::ProgressCallback);

    match FastScaler::process_video(&validated_input, &validated_output, &config, progress_callback) {
        Ok(_) => Ok(output_path),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri command to cancel video processing
#[tauri::command]
async fn cancel_video_processing() -> std::result::Result<bool, String> {
    use crate::core::fast_scaler::cancel_processing;

    let cancelled = cancel_processing();

    // Give FFmpeg a moment to release the file
    if cancelled {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(cancelled)
}

/// Tauri command to delete a file (used for cleanup on cancel)
#[tauri::command]
async fn delete_file(path: String) -> std::result::Result<(), String> {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    // First cancel any running FFmpeg process
    use crate::core::fast_scaler::cancel_processing;
    cancel_processing();

    // SECURITY: Validate that the path is safe for deletion
    let file_path = PathBuf::from(&path);
    let validated_path = validate_deletion_path(&file_path)
        .map_err(|e| format!("Path validation failed: {}", e))?;

    // Try to delete the file with retries
    for attempt in 0..10 {
        // Check if file exists
        if !validated_path.exists() {
            return Ok(());
        }

        // Try to delete
        match fs::remove_file(&validated_path) {
            Ok(_) => {
                return Ok(());
            }
            Err(_e) if attempt < 9 => {
                // File might still be in use by FFmpeg, wait and retry
                thread::sleep(Duration::from_millis(500));
            }
            Err(e) => {
                return Err(format!("Failed to delete file after 10 attempts: {}", e));
            }
        }
    }

    Ok(())
}

/// Tauri command to get current language code
#[tauri::command]
async fn get_current_language() -> String {
    crate::i18n::get_language().code().to_string()
}

/// Tauri command to set current language
#[tauri::command]
async fn set_language_command(lang_code: String) -> std::result::Result<(), String> {
    let lang = Language::from_code(&lang_code)
        .ok_or_else(|| format!("Invalid language code: {}", lang_code))?;
    save_language_preference(lang)
}

/// Tauri command to get all translations for current language
#[tauri::command]
async fn get_translations() -> std::result::Result<serde_json::Value, String> {
    get_all_translations()
}

/// Tauri command to get available languages
#[tauri::command]
async fn get_available_languages() -> Vec<(String, String)> {
    vec![
        ("en".to_string(), "English".to_string()),
        ("zh".to_string(), "中文".to_string()),
    ]
}

/// Tauri command to open a file in its parent folder
#[tauri::command]
async fn open_in_folder(path: String) -> std::result::Result<(), String> {
    use std::process::Command;

    let path_buf = PathBuf::from(&path);

    // SECURITY: Validate that the path is safe
    let validated_path = validate_file_path(&path_buf)
        .map_err(|e| format!("Invalid path: {}", e))?;

    // Check if file exists
    if !validated_path.exists() {
        return Err("File does not exist".to_string());
    }

    // Platform-specific commands to reveal file in folder
    #[cfg(target_os = "windows")]
    {
        // SECURITY: Use proper escaping for Windows explorer
        // Pass /select and the path as separate args to avoid quoting issues
        Command::new("explorer")
            .arg("/select,")
            .arg(&validated_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS open -R is safe with PathBuf as it passes args separately
        Command::new("open")
            .arg("-R")
            .arg(&validated_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // SECURITY: Use file:// URI with proper escaping
        let path_str = validated_path.to_string_lossy();
        // URL-encode the path for the URI
        let encoded_path = urlencoding::encode(&path_str);

        // Try dbus call for file managers that support it (Nautilus, etc.)
        let dbus_result = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:file://{}", encoded_path),
                "string:",
            ])
            .spawn();

        if dbus_result.is_err() {
            // Fallback to opening parent directory
            if let Some(parent) = validated_path.parent() {
                Command::new("xdg-open")
                    .arg(parent)
                    .spawn()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }
        }
    }

    Ok(())
}

/// Tauri command to clean up temporary sample files
#[tauri::command]
async fn cleanup_temp_files(files: Vec<String>) -> std::result::Result<usize, String> {
    use std::fs;

    let mut deleted_count = 0;

    for file_path in files {
        let path = PathBuf::from(&file_path);

        // SECURITY: Validate that the path is safe for deletion
        let validated_path = match validate_deletion_path(&path) {
            Ok(validated) => validated,
            Err(_e) => {
                continue;
            }
        };

        if validated_path.exists() {
            match fs::remove_file(&validated_path) {
                Ok(_) => {
                    deleted_count += 1;
                }
                Err(_) => {
                    // Silently fail for cleanup operations
                }
            }
        }
    }

    Ok(deleted_count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize i18n
            crate::i18n::init();
            println!("Application initialized with language: {}", crate::i18n::get_language().code());

            // Add cleanup handler for window close
            let app_handle = app.handle().clone();
            app.get_webview_window("main").unwrap().on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    println!("Window closing - cleaning up FFmpeg process...");
                    // Cancel any running FFmpeg process
                    let _ = crate::core::fast_scaler::cancel_processing();
                    // Allow the window to close
                    app_handle.get_webview_window("main").unwrap().close().ok();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            process_video,
            generate_sample,
            cancel_video_processing,
            delete_file,
            get_current_language,
            set_language_command,
            get_translations,
            get_available_languages,
            open_in_folder,
            cleanup_temp_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Keep the default greet command for testing
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
