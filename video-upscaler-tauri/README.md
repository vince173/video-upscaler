# Video Upscaler Tauri - Frontend & Tauri Configuration

This is the main application directory for the Video Resolution Upscaler, containing the frontend UI and Tauri backend configuration.

## Quick Start

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server on http://localhost:1420 |
| `npm run build` | Build frontend for production |
| `npm run preview` | Preview production build |
| `npm run tauri dev` | Run full Tauri app in development mode |
| `npm run tauri dev` | Run Tauri dev command |
| `npm run test` | Run E2E tests |
| `npm run test:e2e` | Run WebdriverIO E2E tests |
| `npm run test:e2e:ui` | Run tests with timeout UI |
| `npm run test:e2e:debug` | Run tests in debug mode |
| `npm run test:e2e:watch` | Run tests in watch mode |
| `npm run test:fixtures` | Generate test video fixtures |

## Project Structure

```
video-upscaler-tauri/
├── src/                    # Frontend source (TypeScript)
│   ├── main.ts            # Application entry point
│   ├── styles.css         # Global styles
│   └── assets/            # Static assets
├── src-tauri/             # Tauri backend (Rust)
│   ├── src/
│   │   ├── core/         # Video processing logic
│   │   ├── i18n.rs       # Internationalization
│   │   ├── error.rs      # Error handling
│   │   └── main.rs       # Backend entry point
│   ├── i18n/             # Translation JSON files
│   ├── icons/            # Application icons
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── tests/                # E2E tests
│   └── e2e/
│       ├── fixtures/     # Test video generation
│       ├── specs/        # Test specifications
│       └── helpers/      # Test utilities
├── package.json          # Node.js dependencies
├── vite.config.ts        # Vite bundler configuration
├── tsconfig.json         # TypeScript configuration
├── playwright.config.ts  # Playwright E2E test config
└── wdio.conf.ts          # WebdriverIO E2E test config
```

## Frontend Development

The frontend is built with vanilla TypeScript and Vite:

- **Entry Point**: `src/main.ts`
- **Styling**: `src/styles.css` (CSS with custom properties)
- **Bundler**: Vite 6.x
- **TypeScript**: 5.6.x

### Adding New Features

1. Add UI elements to `index.html`
2. Add styles to `src/styles.css`
3. Add logic to `src/main.ts`

## Backend Development

The Rust backend handles video processing and system operations:

- **Video Processing**: `src-tauri/src/core/fast_scaler.rs`
- **Configuration**: `src-tauri/src/core/config.rs`
- **i18n**: `src-tauri/src/i18n.rs`
- **Errors**: `src-tauri/src/error.rs`

### Adding Tauri Commands

Add commands to `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    // Your logic here
    Ok("result".to_string())
}
```

Then register in `main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    my_command,
    // ... other commands
])
```

## Testing

### E2E Tests

The project uses both WebdriverIO and Playwright for E2E testing:

```bash
# Generate test fixtures (sample videos)
npm run test:fixtures

# Run all E2E tests
npm run test:e2e

# Debug tests
npm run test:e2e:debug
```

### Test Structure

- `tests/e2e/fixtures/` - Test video generation utilities
- `tests/e2e/specs/` - Test specifications
- `tests/e2e/helpers/` - Test helper functions

## Internationalization

Add new languages in `src-tauri/i18n/`:

1. Create `{locale}.json` (e.g., `ja.json`)
2. Add translations for all keys
3. Update `src-tauri/src/i18n.rs` with the new language

## Recommended IDE Setup

- **VS Code** + [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- **Extension**: `Syntax Highlighter` for better code readability
