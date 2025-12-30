# Video Resolution Upscaler

A cross-platform desktop application for upscaling video resolution using FFmpeg. Built with [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), and [TypeScript](https://www.typescriptlang.org/).

## Features

- **Fast Video Upscaling**: Uses FFmpeg for efficient video resolution upscaling
- **Hardware Acceleration**: Supports AMD AMF, NVIDIA NVENC, and Intel QSV hardware encoders
- **Real-time Progress Tracking**: Monitor processing progress with frame-by-frame updates
- **Cancellation Support**: Cancel video processing at any time
- **Multi-language Support**: Available in English and Chinese (Simplified)
- **Cross-platform**: Works on Windows, macOS, and Linux

## Architecture

### Frontend
- **Framework**: Vanilla TypeScript with Vite
- **UI**: Custom HTML/CSS with responsive design
- **State Management**: Simple event-driven architecture

### Backend
- **Framework**: Tauri 2.0
- **Language**: Rust
- **Video Processing**: FFmpeg (via `ffmpeg-sidecar`)
- **Async Runtime**: Tokio

## Project Structure

```
VideoResolutionUpscaler/
├── video-upscaler-tauri/          # Main application directory
│   ├── src/                       # Frontend source code
│   │   ├── main.ts               # Application entry point
│   │   ├── styles.css            # Global styles
│   │   └── assets/               # Static assets
│   ├── src-tauri/                # Tauri/Rust backend
│   │   ├── src/
│   │   │   ├── core/            # Video processing logic
│   │   │   │   ├── fast_scaler.rs  # FFmpeg integration
│   │   │   │   ├── config.rs      # Configuration
│   │   │   │   └── mod.rs         # Module exports
│   │   │   ├── i18n.rs          # Internationalization
│   │   │   ├── error.rs         # Error types
│   │   │   ├── lib.rs           # Library exports
│   │   │   └── main.rs          # Application entry
│   │   ├── i18n/                # Translation files
│   │   ├── icons/               # Application icons
│   │   ├── capabilities/        # Tauri capabilities
│   │   └── tauri.conf.json      # Tauri configuration
│   ├── tests/                   # E2E tests
│   │   └── e2e/
│   │       ├── fixtures/        # Test video generation
│   │       ├── specs/           # Test specifications
│   │       └── helpers/         # Test utilities
│   ├── package.json             # Node dependencies
│   ├── vite.config.ts           # Vite configuration
│   └── tsconfig.json            # TypeScript configuration
└── .github/workflows/           # CI/CD workflows
```

## Getting Started

### Prerequisites

- **Node.js** >= 18.x
- **Rust** >= 1.70
- **FFmpeg** (auto-downloaded by `ffmpeg-sidecar`)
- **System dependencies** for Tauri (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Installation

1. Clone the repository:
```bash
git clone https://github.com/vince173/video-upscaler.git
cd video-upscaler/video-upscaler-tauri
```

2. Install dependencies:
```bash
npm install
```

3. Run in development mode:
```bash
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

The compiled application will be in `src-tauri/target/release/bundle/`.

## Development

### Available Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server |
| `npm run build` | Build frontend for production |
| `npm run tauri dev` | Run Tauri app in development mode |
| `npm run tauri build` | Build Tauri app for production |
| `npm run test` | Run E2E tests |
| `npm run test:e2e` | Run WebdriverIO E2E tests |
| `npm run test:e2e:debug` | Run tests in debug mode |
| `npm run test:fixtures` | Generate test video fixtures |

### Testing

The project includes E2E tests using WebdriverIO and Playwright:

```bash
# Run all tests
npm run test

# Run tests in debug mode
npm run test:e2e:debug

# Generate test video fixtures
npm run test:fixtures
```

### Adding New Languages

1. Create a new JSON file in `src-tauri/i18n/` (e.g., `ja.json`)
2. Add translations following the existing format
3. Update `src/i18n.rs` to include the new language

## Hardware Acceleration

The application supports hardware-accelerated encoding for faster processing:

| Platform | Encoder | Status |
|----------|---------|--------|
| Windows | AMD AMF | Supported |
| Windows | NVIDIA NVENC | Supported |
| Windows | Intel QSV | Supported |
| macOS | VideoToolbox | Planned |
| Linux | VAAPI | Planned |

## Configuration

Video processing can be configured through the `Config` struct in `src-tauri/src/core/config.rs`:

- **Scale Factor**: Output resolution multiplier (default: 4x)
- **Quality**: Encoding quality preset
- **Encoder**: Hardware encoder selection
- **Mode**: Processing mode (Fast/Quality)

## Troubleshooting

### FFmpeg not found
The application uses `ffmpeg-sidecar` which automatically downloads FFmpeg. If you encounter issues, manually download FFmpeg and ensure it's in your PATH.

### Build errors on Windows
Make sure you have:
- Visual Studio C++ Build Tools
- Rust installed via rustup
- WebView2 runtime (usually pre-installed on Windows 10+)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License.

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [FFmpeg](https://ffmpeg.org/) - Video processing library
- [ffmpeg-sidecar](https://github.com/nathanabram/github-ffmpeg-sidecar) - FFmpeg Rust bindings
- [WebdriverIO](https://webdriver.io/) - E2E testing framework
