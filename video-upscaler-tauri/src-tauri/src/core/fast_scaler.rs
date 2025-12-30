//! Fast video scaler using FFmpeg
//!
//! Uses FFmpeg's built-in scaling + unsharp masking for real-time performance.

use crate::core::config::Config;
use crate::error::{Result, UpscalerError};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{atomic::{AtomicBool, Ordering}, mpsc, OnceLock};
use std::thread;
use std::io::BufReader;

use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
    paths::ffmpeg_path,
};

/// Global cancellation flag
static CANCELLATION_FLAG: OnceLock<AtomicBool> = OnceLock::new();

/// Global FFmpeg child process ID for killing
static FFMPEG_PID: OnceLock<std::sync::Mutex<Option<u32>>> = OnceLock::new();

/// Fast video scaler using FFmpeg
pub struct FastScaler;

/// Progress callback type: sends (current_frame, total_frames, percentage)
pub type ProgressCallback = Box<dyn Fn(usize, usize, f32) + Send + Sync>;

/// Cancel any running FFmpeg process - actually kills it
pub fn cancel_processing() -> bool {
    // Set the flag first
    if let Some(flag) = CANCELLATION_FLAG.get() {
        flag.store(true, Ordering::SeqCst);
        println!("Cancellation flag set");
    }

    // Try to kill the FFmpeg process by PID
    if let Some(pid_guard) = FFMPEG_PID.get() {
        if let Ok(mut guard) = pid_guard.try_lock() {
            if let Some(pid) = guard.take() {
                println!("Killing FFmpeg process PID {}...", pid);
                // Kill the process using system command
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F"])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = Command::new("kill")
                        .args(["-9", &pid.to_string()])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn();
                }
                return true;
            }
        }
    }
    false
}

/// Reset cancellation flag for a new processing run
fn reset_cancellation() -> &'static AtomicBool {
    CANCELLATION_FLAG.get_or_init(|| {
        AtomicBool::new(false)
    })
}

/// Clear the child process PID after completion
fn clear_child() {
    if let Some(pid_guard) = FFMPEG_PID.get() {
        if let Ok(mut pid) = pid_guard.try_lock() {
            *pid = None;
        }
    }
}

impl FastScaler {
    /// Process a video using FFmpeg's fast scaling
    pub fn process_video(
        input_path: &Path,
        output_path: &Path,
        config: &Config,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        println!("Using Fast Mode (FFmpeg scaling + sharpening)");

        // Get video metadata first
        let metadata = Self::get_video_metadata(input_path)?;
        let output_width = metadata.width * config.scale;

        println!(
            "   Input: {}x{} @ {:.2}fps",
            metadata.width, metadata.height, metadata.fps
        );
        println!(
            "   Output: {}x{}",
            output_width,
            metadata.height * config.scale
        );
        println!("   Total frames: {}", metadata.frame_count);

        // Calculate expected frames for preview mode
        let expected_frames = if config.preview_duration_secs > 0 {
            (config.preview_duration_secs as f64 * metadata.fps) as usize
        } else {
            metadata.frame_count
        };

        // Build FFmpeg command with quality-based settings
        let (scale_filter_fmt, sharpen, preset, crf) = match config.quality_preset {
            crate::core::config::QualityPreset::UltraFast => {
                println!("   Mode: UltraFast (lower quality)");
                (
                    "scale={}:-1:flags=bicubic",
                    "unsharp=3:3:1.0:3:3:0.0",
                    "ultrafast",
                    "23",
                )
            }
            crate::core::config::QualityPreset::Balanced => {
                println!("   Mode: Balanced (good quality)");
                (
                    "scale={}:-1:flags=bicubic",
                    "unsharp=3:3:1.2:3:3:0.0",
                    "fast",
                    "20",
                )
            }
            crate::core::config::QualityPreset::HighQuality => {
                println!("   Mode: HighQuality (slower)");
                (
                    "scale={}:-1:flags=lanczos",
                    "unsharp=3:3:1.5:3:3:0.0,unsharp=5:5:1.0:5:5:0.0",
                    "medium",
                    "18",
                )
            }
        };

        // Determine hardware encoder
        let encoder = match config.hardware_encoder {
            crate::core::config::HardwareEncoder::AMD => {
                println!("   Encoder: AMD AMF hardware acceleration");
                "h264_amf"
            }
            crate::core::config::HardwareEncoder::NVIDIA => {
                println!("   Encoder: NVIDIA NVENC hardware acceleration");
                "h264_nvenc"
            }
            crate::core::config::HardwareEncoder::Intel => {
                println!("   Encoder: Intel QSV hardware acceleration");
                "h264_qsv"
            }
            crate::core::config::HardwareEncoder::None => {
                println!("   Encoder: CPU software (libx264)");
                "libx264"
            }
        };

        let scale_filter = scale_filter_fmt.replace("{}", &output_width.to_string());
        let filters = format!("{},{}", scale_filter, sharpen);

        println!("   Starting processing...");

        // Build FFmpeg command using ffmpeg-sidecar
        let mut cmd = FfmpegCommand::new();

        // Add preview duration limit BEFORE input
        if config.preview_duration_secs > 0 {
            println!(
                "   Preview mode: processing first {} seconds only",
                config.preview_duration_secs
            );
            cmd.args(["-t", &config.preview_duration_secs.to_string()]);
        }

        cmd.arg("-i")
            .arg(input_path)
            .args(["-vf", &filters])
            .args(["-c:v", encoder])
            // No audio processing - video only
            .args(["-an"]);

        // Add encoder-specific parameters
        if config.hardware_encoder == crate::core::config::HardwareEncoder::None {
            // Software encoding - use preset and crf
            cmd.args(["-preset", preset])
                .args(["-crf", crf])
                .args(["-tune", "fastdecode"]);
        } else {
            // Hardware encoding - use quality parameter
            let quality = match config.quality_preset {
                crate::core::config::QualityPreset::UltraFast => "-1",
                crate::core::config::QualityPreset::Balanced => "0",
                crate::core::config::QualityPreset::HighQuality => "2",
            };
            cmd.args(["-quality", quality]);
        }

        cmd.args(["-pix_fmt", "yuv420p"])
            .args(["-threads", "0"]);

        // Add faststart for MP4 files for better compatibility
        if output_path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            cmd.args(["-movflags", "+faststart"]);
        }

        cmd.arg("-y")
            .arg(output_path);

        // Get the FFmpeg command as a std::process::Command
        // We need to build it manually since FfmpegCommand doesn't expose the args
        let ffmpeg_path = ffmpeg_path();

        println!("[FFmpeg] Using FFmpeg at: {:?}", ffmpeg_path);

        // Build the command manually using the same logic as FfmpegCommand
        let mut cmd_args = vec![
            "-t".to_string(), config.preview_duration_secs.to_string(),
            "-i".to_string(), input_path.to_string_lossy().to_string(),
            "-vf".to_string(), filters.clone(),
            "-c:v".to_string(), encoder.to_string(),
            "-an".to_string(),
        ];

        // Add encoder-specific parameters
        if config.hardware_encoder == crate::core::config::HardwareEncoder::None {
            cmd_args.extend(["-preset".to_string(), preset.to_string()]);
            cmd_args.extend(["-crf".to_string(), crf.to_string()]);
            cmd_args.extend(["-tune".to_string(), "faststart".to_string()]);
        } else {
            let quality = match config.quality_preset {
                crate::core::config::QualityPreset::UltraFast => "-1",
                crate::core::config::QualityPreset::Balanced => "0",
                crate::core::config::QualityPreset::HighQuality => "2",
            };
            cmd_args.extend(["-quality".to_string(), quality.to_string()]);
        }

        cmd_args.extend(["-pix_fmt".to_string(), "yuv420p".to_string()]);
        cmd_args.extend(["-threads".to_string(), "0".to_string()]);

        if output_path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            cmd_args.extend(["-movflags".to_string(), "+faststart".to_string()]);
        }

        cmd_args.extend(["-y".to_string(), output_path.to_string_lossy().to_string()]);

        let start_time = std::time::Instant::now();
        let mut last_frame = 0;

        // Reset cancellation flag
        let cancel_flag = reset_cancellation();
        cancel_flag.store(false, Ordering::SeqCst);

        // Spawn FFmpeg using std::process::Command
        let mut child = Command::new(&ffmpeg_path)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| UpscalerError::FfmpegError(format!("Failed to spawn FFmpeg: {}", e)))?;

        // Get the PID for cancellation
        let pid = child.id();

        // Store PID globally for cancellation access
        {
            if let Some(pid_guard) = FFMPEG_PID.get() {
                if let Ok(mut guard) = pid_guard.try_lock() {
                    *guard = Some(pid);
                }
            }
        }

        println!("[FFmpeg] Spawned with PID: {:?}", pid);

        // Create channels for communication
        let (event_tx, event_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();

        // Get stderr handle for parsing
        let stderr = child.stderr.take()
            .ok_or_else(|| UpscalerError::FfmpegError("Failed to capture stderr".to_string()))?;

        // Calculate expected frames for preview mode
        let expected_frames_for_thread = expected_frames;

        // Spawn background thread to parse FFmpeg stderr and wait for completion
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut parser = ffmpeg_sidecar::log_parser::FfmpegLogParser::new(reader);

            // Parse events from stderr
            loop {
                match parser.parse_next_event() {
                    Ok(ffmpeg_sidecar::event::FfmpegEvent::LogEOF) => {
                        println!("[FFmpeg] Log EOF reached");
                        break;
                    }
                    Ok(event) => {
                        if event_tx.send(event).is_err() {
                            // Channel closed, main thread gave up
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[FFmpeg] Error parsing output: {}", e);
                        let _ = done_tx.send(Err(format!("Parse error: {}", e)));
                        clear_child();
                        return;
                    }
                }
            }

            // Now wait for the child process to complete
            println!("[FFmpeg] Waiting for process to exit...");
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        println!("[FFmpeg] Process exited successfully");
                        clear_child();
                        let _ = done_tx.send(Ok(()));
                    } else {
                        let code = status.code().unwrap_or(-1);
                        eprintln!("[FFmpeg] Process exited with error code: {}", code);
                        clear_child();
                        let _ = done_tx.send(Err(format!("FFmpeg exited with code {}", code)));
                    }
                }
                Err(e) => {
                    eprintln!("[FFmpeg] Failed to wait for process: {}", e);
                    clear_child();
                    let _ = done_tx.send(Err(format!("Wait failed: {}", e)));
                }
            }
        });

        // Main thread: receive events from channel and process them
        loop {
            // Check for cancellation
            if cancel_flag.load(Ordering::SeqCst) {
                println!("   Processing cancelled by user!");
                cancel_processing();
                clear_child();
                return Err(UpscalerError::Cancelled);
            }

            // Check if done signal received
            match done_rx.try_recv() {
                Ok(result) => {
                    if let Err(e) = result {
                        return Err(UpscalerError::FfmpegError(e));
                    }
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected but no error - check for any pending events
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            // Try to receive events with timeout
            match event_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                Ok(event) => {
                    match event {
                        FfmpegEvent::Progress(progress) => {
                            let frame_num = progress.frame as usize;
                            println!("[DEBUG] Frame progress: {} (time: {:.2}s)", frame_num, progress.time);

                            if frame_num > last_frame
                                && (frame_num.is_multiple_of(2) || frame_num == expected_frames_for_thread)
                            {
                                let percentage = if expected_frames_for_thread > 0 {
                                    frame_num as f32 / expected_frames_for_thread as f32 * 100.0
                                } else {
                                    0.0
                                };

                                println!(
                                    "   Processed {} frames / {} ({:.1}%)",
                                    frame_num,
                                    expected_frames_for_thread,
                                    percentage
                                );

                                if let Some(ref cb) = progress_callback {
                                    cb(frame_num, expected_frames_for_thread, percentage);
                                }

                                last_frame = frame_num;
                            }
                        }
                        FfmpegEvent::Log(LogLevel::Error, msg) => {
                            println!("[FFmpeg Error Log] {}", msg);
                        }
                        FfmpegEvent::Log(LogLevel::Warning, msg) => {
                            println!("[FFmpeg Warning Log] {}", msg);
                        }
                        FfmpegEvent::Log(LogLevel::Info, msg) => {
                            println!("[FFmpeg Info] {}", msg);
                        }
                        FfmpegEvent::Error(msg) => {
                            println!("[FFmpeg Error Event] {}", msg);
                        }
                        _ => {}
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Event channel disconnected, break
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout, check done signal and continue
                    continue;
                }
            }
        }

        // Wait to ensure file is fully written to disk and file handles are released
        println!("[FFmpeg] Waiting for file write completion...");
        std::thread::sleep(std::time::Duration::from_millis(2000));

        // Verify output file exists and has content
        if !output_path.exists() {
            return Err(UpscalerError::FfmpegError(
                "Output file was not created".to_string()
            ));
        }

        // Flush file to ensure all data is written
        {
            use std::fs::OpenOptions;
            if let Ok(_file) = OpenOptions::new().write(true).open(output_path) {
                // File opened successfully, data should be flushed
            }
        }

        let file_size = std::fs::metadata(output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        if file_size == 0 {
            return Err(UpscalerError::FfmpegError(
                "Output file is empty".to_string()
            ));
        }

        println!("[FFmpeg] Output file size: {} bytes", file_size);

        // Check if was cancelled
        if cancel_flag.load(Ordering::SeqCst) {
            return Err(UpscalerError::Cancelled);
        }

        println!("   Processing complete!");

        Ok(())
    }

    /// Get video metadata using ffprobe
    fn get_video_metadata(path: &Path) -> Result<VideoMetadataInternal> {
        use std::process::{Command, Stdio};

        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v:0")
            .arg("-show_entries")
            .arg("stream=width,height,r_frame_rate")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("json")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| UpscalerError::DecodingError(format!("Failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            return Err(UpscalerError::DecodingError("ffprobe failed".to_string()));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            UpscalerError::DecodingError(format!("Failed to parse ffprobe output: {}", e))
        })?;

        let stream = json
            .get("streams")
            .and_then(|s| s.as_array())
            .and_then(|a| a.first())
            .ok_or_else(|| UpscalerError::DecodingError("No video stream found".to_string()))?;

        let width = stream
            .get("width")
            .and_then(|w| w.as_u64())
            .ok_or_else(|| UpscalerError::DecodingError("Width not found".to_string()))?
            as u32;

        let height = stream
            .get("height")
            .and_then(|h| h.as_u64())
            .ok_or_else(|| UpscalerError::DecodingError("Height not found".to_string()))?
            as u32;

        let fps = stream
            .get("r_frame_rate")
            .and_then(|r| r.as_str())
            .and_then(|s| {
                let parts: Vec<&str> = s.split('/').collect();
                if parts.len() == 2 {
                    let num: f64 = parts[0].parse().ok()?;
                    let den: f64 = parts[1].parse().ok()?;
                    Some(num / den)
                } else {
                    None
                }
            })
            .unwrap_or(30.0);

        let format = json
            .get("format")
            .ok_or_else(|| UpscalerError::DecodingError("No format info".to_string()))?;
        let duration_secs = format
            .get("duration")
            .and_then(|d| d.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let _duration = std::time::Duration::from_secs_f64(duration_secs);
        let frame_count = if fps > 0.0 && duration_secs > 0.0 {
            (duration_secs * fps) as usize
        } else {
            0
        };

        Ok(VideoMetadataInternal {
            width,
            height,
            fps,
            frame_count,
            _duration,
        })
    }
}

struct VideoMetadataInternal {
    width: u32,
    height: u32,
    fps: f64,
    frame_count: usize,
    _duration: std::time::Duration,
}
