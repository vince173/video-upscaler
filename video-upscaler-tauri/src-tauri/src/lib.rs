mod constants;
mod core;
mod error;
mod i18n;
mod security;

use crate::constants::*;
use crate::core::fast_scaler::{FastScaler, ProgressCallback};
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

/// Creates a progress callback that emits Tauri events to the frontend.
///
/// # Arguments
/// * `app` - The Tauri AppHandle used to emit events
///
/// # Returns
/// An optional ProgressCallback that emits video-progress events
fn create_progress_callback(app: AppHandle) -> Option<ProgressCallback> {
    let app_handle = Arc::new(app);
    Some(Box::new(move |current: usize, total: usize, percentage: f32| {
        let _ = app_handle.emit("video-progress", serde_json::json!({
            "current": current,
            "total": total,
            "percentage": percentage
        }));
    }))
}

/// Validates and prepares video processing paths.
///
/// # Arguments
/// * `input_path` - The input video file path
/// * `output_path` - The output video file path
///
/// # Returns
/// A Result containing validated (input, output) PathBuf tuple or an error message
fn validate_video_paths(input_path: String, output_path: String) -> std::result::Result<(PathBuf, PathBuf), String> {
    let input = PathBuf::from(&input_path);
    let validated_input = validate_video_file(&input)
        .map_err(|e| format!("Input validation failed: {}", e))?;

    let output = PathBuf::from(&output_path);
    let validated_output = validate_output_path(&output)
        .map_err(|e| format!("Output validation failed: {}", e))?;

    Ok((validated_input, validated_output))
}

/// Parses a quality preset string into a QualityPreset enum.
///
/// # Arguments
/// * `quality` - The quality string (e.g., "ultra_fast", "balanced", "high_quality")
///
/// # Returns
/// The corresponding QualityPreset, defaulting to Balanced if unknown
fn parse_quality_preset(quality: &str) -> QualityPreset {
    match quality {
        quality::ULTRA_FAST => QualityPreset::UltraFast,
        quality::BALANCED => QualityPreset::Balanced,
        quality::HIGH_QUALITY => QualityPreset::HighQuality,
        _ => QualityPreset::Balanced,
    }
}

/// Parses a hardware encoder string into a HardwareEncoder enum.
///
/// # Arguments
/// * `encoder` - The encoder string (e.g., "amd", "nvidia", "intel")
///
/// # Returns
/// The corresponding HardwareEncoder, defaulting to None if unknown
fn parse_hardware_encoder(encoder: &str) -> HardwareEncoder {
    match encoder {
        encoder::AMD => HardwareEncoder::AMD,
        encoder::NVIDIA => HardwareEncoder::NVIDIA,
        encoder::INTEL => HardwareEncoder::Intel,
        _ => HardwareEncoder::None,
    }
}

/// Attempts to delete a file with retry logic for handling temporary locks.
///
/// # Arguments
/// * `path` - The path to delete
/// * `max_retries` - Maximum number of retry attempts
/// * `retry_delay_ms` - Delay between retries in milliseconds
///
/// # Returns
/// Ok(()) if deleted or doesn't exist, Err with message if all retries fail
fn delete_file_with_retry(path: &PathBuf, max_retries: u32, retry_delay_ms: u64) -> std::result::Result<(), String> {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    for attempt in 0..max_retries {
        if !path.exists() {
            return Ok(());
        }

        match fs::remove_file(path) {
            Ok(_) => return Ok(()),
            Err(_) if attempt < max_retries - 1 => {
                thread::sleep(Duration::from_millis(retry_delay_ms));
            }
            Err(e) => return Err(format!("Failed to delete file after {} attempts: {}", max_retries, e)),
        }
    }

    Ok(())
}

/// Tauri command to process a video file with specified scaling, quality, and encoder options.
///
/// # Arguments
/// * `app` - Tauri AppHandle for emitting progress events
/// * `input_path` - Path to the input video file
/// * `output_path` - Path where the output video will be saved
/// * `scale` - Optional scale factor (e.g., 2 for 2x upscale)
/// * `quality` - Optional quality preset: "ultra_fast", "balanced", or "high_quality"
/// * `hardware_encoder` - Optional hardware encoder: "amd", "nvidia", "intel", or "none"
///
/// # Returns
/// Ok(output_path) on success, Err(error_message) on failure
#[tauri::command]
async fn process_video(
    app: AppHandle,
    input_path: String,
    output_path: String,
    scale: Option<u32>,
    quality: Option<String>,
    hardware_encoder: Option<String>,
) -> std::result::Result<String, String> {
    ensure_ffmpeg()?;

    let (validated_input, validated_output) = validate_video_paths(input_path, output_path)?;

    let mut config = Config::new();
    if let Some(s) = scale {
        config = config.with_scale(s);
    }
    if let Some(q) = quality {
        config = config.with_quality_preset(parse_quality_preset(&q));
    }
    if let Some(he) = hardware_encoder {
        config = config.with_hardware_encoder(parse_hardware_encoder(&he));
    }

    let progress_callback = create_progress_callback(app);

    match FastScaler::process_video(&validated_input, &validated_output, &config, progress_callback) {
        Ok(_) => Ok(validated_output.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri command to generate a short preview sample from a video.
///
/// Creates a 10-second sample with 4x scaling for quality preview.
///
/// # Arguments
/// * `app` - Tauri AppHandle for emitting progress events
/// * `input_path` - Path to the input video file
/// * `output_path` - Path where the sample will be saved
///
/// # Returns
/// Ok(output_path) on success, Err(error_message) on failure
#[tauri::command]
async fn generate_sample(
    app: AppHandle,
    input_path: String,
    output_path: String,
) -> std::result::Result<String, String> {
    ensure_ffmpeg()?;

    let (validated_input, validated_output) = validate_video_paths(input_path, output_path)?;

    let config = Config::new()
        .with_scale(DEFAULT_SAMPLE_SCALE)
        .with_preview_duration(DEFAULT_SAMPLE_DURATION_SECS);

    let progress_callback = create_progress_callback(app);

    match FastScaler::process_video(&validated_input, &validated_output, &config, progress_callback) {
        Ok(_) => Ok(validated_output.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Tauri command to cancel an ongoing video processing operation.
///
/// # Returns
/// Ok(true) if cancellation was initiated, Ok(false) if no process was running
#[tauri::command]
async fn cancel_video_processing() -> std::result::Result<bool, String> {
    use crate::core::fast_scaler::cancel_processing;

    let cancelled = cancel_processing();

    if cancelled {
        std::thread::sleep(std::time::Duration::from_millis(CANCELLATION_GRACE_PERIOD_MS));
    }

    Ok(cancelled)
}

/// Tauri command to delete a file, typically used for cleanup after canceling processing.
///
/// Cancels any running FFmpeg process first, then deletes the specified file with retry logic.
///
/// # Arguments
/// * `path` - Path to the file to delete
///
/// # Returns
/// Ok(()) on success, Err(error_message) on failure
#[tauri::command]
async fn delete_file(path: String) -> std::result::Result<(), String> {
    use crate::core::fast_scaler::cancel_processing;

    cancel_processing();

    let file_path = PathBuf::from(&path);
    let validated_path = validate_deletion_path(&file_path)
        .map_err(|e| format!("Path validation failed: {}", e))?;

    delete_file_with_retry(&validated_path, FILE_DELETE_RETRY_COUNT, FILE_DELETE_RETRY_DELAY_MS)
}

/// Tauri command to get the current language code.
///
/// # Returns
/// The ISO language code (e.g., "en", "zh")
#[tauri::command]
fn get_current_language() -> &'static str {
    crate::i18n::get_language().code()
}

/// Tauri command to set the current application language.
///
/// # Arguments
/// * `lang_code` - ISO language code (e.g., "en", "zh")
///
/// # Returns
/// Ok(()) on success, Err(error_message) if the language code is invalid
#[tauri::command]
fn set_language(lang_code: String) -> std::result::Result<(), String> {
    let lang = Language::from_code(&lang_code)
        .ok_or_else(|| format!("Invalid language code: {}", lang_code))?;
    save_language_preference(lang)
}

/// Tauri command to get all translations for the current language.
///
/// # Returns
/// A JSON object containing all translation key-value pairs
#[tauri::command]
fn get_translations() -> std::result::Result<serde_json::Value, String> {
    get_all_translations()
}

/// Tauri command to get the list of available languages.
///
/// # Returns
/// A vector of (code, display_name) tuples for each supported language
#[tauri::command]
fn get_available_languages() -> Vec<(String, String)> {
    vec![
        (Language::English.code().to_string(), "English".to_string()),
        (Language::Chinese.code().to_string(), "中文".to_string()),
    ]
}

/// Tauri command to reveal a file in the platform's file manager.
///
/// Opens the parent folder and selects the specified file.
/// On Windows: uses explorer with /select
/// On macOS: uses open -R
/// On Linux: uses dbus-send with fallback to xdg-open
///
/// # Arguments
/// * `path` - Path to the file to reveal
///
/// # Returns
/// Ok(()) on success, Err(error_message) on failure
#[tauri::command]
fn open_in_folder(path: String) -> std::result::Result<(), String> {
    use std::process::Command;

    let path_buf = PathBuf::from(&path);
    let validated_path = validate_file_path(&path_buf)
        .map_err(|e| format!("Invalid path: {}", e))?;

    if !validated_path.exists() {
        return Err("File does not exist".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg("/select,")
            .arg(&validated_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(&validated_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let path_str = validated_path.to_string_lossy();
        let encoded_path = urlencoding::encode(&path_str);

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

/// Tauri command to clean up temporary sample files.
///
/// Attempts to delete all specified files, returning detailed results.
///
/// # Arguments
/// * `files` - List of file paths to delete
///
/// # Returns
/// Ok(deleted_count) - the number of files successfully deleted
#[tauri::command]
fn cleanup_temp_files(files: Vec<String>) -> std::result::Result<usize, String> {
    use std::fs;

    let mut deleted_count = 0;
    let mut failed_files = Vec::new();

    for file_path in files {
        let path = PathBuf::from(&file_path);

        let validated_path = match validate_deletion_path(&path) {
            Ok(validated) => validated,
            Err(_) => continue,
        };

        if validated_path.exists() {
            match fs::remove_file(&validated_path) {
                Ok(_) => deleted_count += 1,
                Err(_) => failed_files.push(file_path),
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
            process_video,
            generate_sample,
            cancel_video_processing,
            delete_file,
            get_current_language,
            set_language,
            get_translations,
            get_available_languages,
            open_in_folder,
            cleanup_temp_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
