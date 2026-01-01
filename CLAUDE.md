# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A cross-platform desktop video upscaling application built with Tauri 2.0, Rust, and TypeScript. Uses FFmpeg for video processing with hardware acceleration support (AMD AMF, NVIDIA NVENC, Intel QSV).

**Working directory**: All development happens in `video-upscaler-tauri/`

## Common Commands

**Development**:
```bash
cd video-upscaler-tauri
npm run tauri dev          # Run dev server with hot reload
npm run dev                # Start Vite dev server only
```

**Building**:
```bash
npm run build              # Build frontend for production
npm run tauri build        # Build full Tauri app for distribution
```

**Testing**:
```bash
npm run test               # Run all E2E tests
npm run test:e2e           # Run WebdriverIO E2E tests
npm run test:e2e:debug     # Run tests in debug mode
npm run test:fixtures      # Generate test video fixtures (FFmpeg required)
```

**Rust-specific**:
```bash
cd src-tauri
cargo check                # Check for compilation errors
cargo clippy               # Run linter
cargo test                 # Run Rust unit tests
cargo fmt                  # Format code
```

**Windows WebDriver for tests**: The `tauri-driver` package uses Edge WebDriver on Windows. Install via `winget install Microsoft.Edge.WebDriver` or let GitHub Actions handle it in CI.

## Architecture

### Frontend (`src/`)
- **Entry point**: `src/main.ts` - Single-file TypeScript application
- **State management**: Simple event-driven architecture with global variables
- **i18n**: Translations loaded from backend via `get_translations()` command
- **Video playback**: Uses HTML5 `<video>` with `convertFileSrc()` for Tauri asset protocol
- **Progress tracking**: Listens to `video-progress` events emitted by Rust backend

### Backend (`src-tauri/src/`)
- **`lib.rs`**: Tauri commands (invoke handlers) - entry points for frontend calls
- **`core/fast_scaler.rs`**: FFmpeg video processing using `ffmpeg-sidecar`
- **`core/config.rs`**: Configuration types (Config, HardwareEncoder, QualityPreset, ProcessingMode)
- **`error.rs`**: Error types (UpscalerError enum)
- **`i18n.rs`**: Internationalization with JSON-based translations in `src-tauri/i18n/`

### Key Tauri Commands (Frontend → Rust)
| Command | Description |
|---------|-------------|
| `process_video` | Process full video with scale/quality/encoder options |
| `generate_sample` | Generate 10-second preview sample |
| `cancel_video_processing` | Cancel running FFmpeg process |
| `delete_file` | Delete partial output file on cancel |
| `get_current_language` | Get current language code |
| `set_language_command` | Change language |
| `get_translations` | Get all translations for current language |
| `open_in_folder` | Reveal file in platform file manager |
| `cleanup_temp_files` | Delete sample files on window close |

### Video Processing Flow
1. Frontend invokes command (e.g., `process_video`)
2. Rust backend spawns FFmpeg process via `ffmpeg-sidecar`
3. Background thread parses FFmpeg stderr for progress events
4. Progress emitted to frontend via `video-progress` Tauri event
5. Frontend updates UI with frame count and percentage
6. Cancellation: Global `CANCELLATION_FLAG` + PID-based process kill

### Cancellation Mechanism
- Global `AtomicBool` flag (`CANCELLATION_FLAG`) checked in processing loop
- Global FFmpeg PID stored in `FFMPEG_PID` for `taskkill`/`kill` commands
- 500ms delay after cancellation to ensure FFmpeg releases file handles
- File deletion retries up to 10 times with 500ms intervals

### Hardware Encoders
- **AMD**: `h264_amf` (Windows)
- **NVIDIA**: `h264_nvenc` (Windows)
- **Intel**: `h264_qsv` (Windows)
- **Software**: `libx264` (all platforms)

### Quality Presets (affect FFmpeg filter graph and encoding parameters)
- **UltraFast**: bicubic scale + light sharpen + ultrafast preset + CRF 23
- **Balanced**: bicubic scale + medium sharpen + fast preset + CRF 20
- **HighQuality**: lanczos scale + heavy sharpen + medium preset + CRF 18

## Testing

**E2E Tests** (`tests/e2e/`):
- WebdriverIO with `@crabnebula/tauri-driver`
- Test fixtures generated via `npm run test:fixtures` (requires FFmpeg in PATH)
- Specs: sample generation, cancellation, i18n

**Adding a new E2E test**:
1. Create spec file in `tests/e2e/specs/`
2. Use `@crabnebula/tauri-driver` to launch app
3. Use browser automation APIs (WebDriver protocol)

## Adding New Translations

1. Create JSON file in `src-tauri/i18n/` (e.g., `ja.json`)
2. Add translations following existing structure in `en.json`/`zh.json`
3. Update `i18n.rs`: Add language variant to `Language` enum, update `code()`, `display_name()`, `from_code()`
4. Update `get_available_languages()` command in `lib.rs`

## Important Implementation Details

- **FFmpeg initialization**: Auto-downloaded via `ffmpeg-sidecar::download::auto_download()` on first use
- **File handle cleanup**: 2 second sleep after FFmpeg completion to ensure file writes finish
- **Sample cleanup**: Frontend tracks sample files in `sampleFiles` array, cleaned up on `beforeunload`
- **Platform-specific code**: `#[cfg(target_os = "windows/mac/linux")]` attributes for file operations

## Development Guidelines

- **Git operations**: ALWAYS ask for user permission before running `git commit`, `git push`, or any other git commands that modify the repository state or history. Never commit or push without explicit user confirmation.
