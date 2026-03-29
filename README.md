# Clipboard Investigator

A desktop app to inspect and analyze clipboard contents in detail. See every data type, preview images, and explore binary payloads.

Built with [Tauri 2](https://tauri.app) (Rust + Vanilla JS). Works on macOS and Windows.

**[Try it in your browser](https://vijaykrishnavanshi.github.io/clipboard-investigator/)**

## Download

Grab the latest release for your platform:

| Platform | Download |
|----------|----------|
| macOS (Universal) | [`.dmg`](https://github.com/vijaykrishnavanshi/clipboard-investigator/releases/latest) |
| Windows | [`.msi` / `.exe`](https://github.com/vijaykrishnavanshi/clipboard-investigator/releases/latest) |

See all versions on the [Releases](https://github.com/vijaykrishnavanshi/clipboard-investigator/releases) page.

## Features

- **Read Clipboard** -- Click a button or press `Cmd+V` / `Ctrl+V` to read all clipboard types via native OS APIs
- **Drag & Drop** -- Drop files into the window to inspect their DataTransfer details
- **Image Preview** -- Detects image data (PNG, JPEG, TIFF, etc.) and shows inline thumbnails
- **Binary Data** -- Non-text entries are base64-encoded and displayed with size info
- **Type Identification** -- Shows the UTI (macOS) or format name (Windows) for each entry
- **CLI Mode** -- Run from the terminal with `--cli`, `--json`, or `--types` flags
- **System Tray** -- Background menu bar app with clipboard preview and quick actions

## CLI Usage

The app binary doubles as a CLI tool:

```sh
clipboard-investigator --cli       # Human-readable clipboard dump
clipboard-investigator --json      # Full JSON output (for scripting)
clipboard-investigator --types     # List type names only
clipboard-investigator --help      # Show help
clipboard-investigator             # Launch GUI (default)
```

## Prerequisites

- **Rust** 1.77.2 or later -- [install via rustup](https://rustup.rs)
- **Tauri CLI** -- install with:

```sh
cargo install tauri-cli --version "^2"
```

## Getting Started

Clone the repository:

```sh
git clone https://github.com/vijaykrishnavanshi/clipboard-investigator.git
cd clipboard-investigator
```

Run in development mode:

```sh
cd src-tauri
cargo tauri dev
```

This launches the app with hot-reload for the frontend. The window opens at 1024x768.

## Building for Production

```sh
cd src-tauri
cargo tauri build
```

The built bundle is output to:

```
# macOS
src-tauri/target/release/bundle/macos/Clipboard Investigator.app
src-tauri/target/release/bundle/dmg/Clipboard Investigator_<version>_universal.dmg

# Windows
src-tauri/target/release/bundle/msi/
src-tauri/target/release/bundle/nsis/
```

## Project Structure

```
clipboard-investigator/
  src-tauri/
    src/
      lib.rs          # Core logic: clipboard reading (macOS/Windows), tray, CLI exports
      main.rs         # Entry point: CLI arg handling or GUI launch
    Cargo.toml        # Rust dependencies
    tauri.conf.json   # Tauri app config (window, CSP, bundling)
    capabilities/     # Tauri permission definitions
    icons/            # App icons (.icns, .ico, .png)
  frontend/
    index.html        # UI with inline CSS/JS, Tauri IPC calls
    logo.svg          # App logo
  docs/
    index.html        # GitHub Pages website (browser-only version)
```

## How It Works

The Rust backend reads clipboard data using platform-native APIs:
- **macOS**: `NSPasteboard` via the `objc` crate (Objective-C interop)
- **Windows**: Win32 `OpenClipboard` / `EnumClipboardFormats` / `GetClipboardData` via the `windows` crate

It iterates all registered clipboard types, classifies each as text or binary, and extracts the data -- text is returned directly, binary data is base64-encoded.

The frontend receives a `Vec<ClipboardEntry>` (serialized as JSON) via Tauri's IPC bridge and renders it as an HTML table with type, content preview, and size columns.

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Desktop Framework | Tauri | 2.3.1 |
| Backend Language | Rust | 1.77.2+ |
| Frontend | Vanilla JS / HTML / CSS | -- |
| macOS Interop | `objc` crate | 0.2 |
| Windows Interop | `windows` crate | 0.58 |
| Serialization | `serde` / `serde_json` | 1.0 |
| Binary Encoding | `base64` crate | 0.22 |

## License

MIT

Based on [clipboard-inspector](https://github.com/evercoder/clipboard-inspector) by [Moqups Labs](https://labs.moqups.com).
