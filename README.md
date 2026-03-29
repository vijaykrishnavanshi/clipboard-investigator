# Clipboard Investigator

A macOS desktop app to inspect and analyze clipboard contents in detail. See every data type, preview images, and explore binary payloads.

Built with [Tauri 2](https://tauri.app) (Rust + Vanilla JS).

**[Try it in your browser](https://vijaykrishnavanshi.github.io/clipboard-investigator/)**

## Features

- **Read Clipboard** -- Click a button or press `Cmd+V` to read all clipboard types via the native NSPasteboard API
- **Drag & Drop** -- Drop files into the window to inspect their DataTransfer details
- **Image Preview** -- Detects image data (PNG, JPEG, TIFF, etc.) and shows inline thumbnails
- **Binary Data** -- Non-text entries are base64-encoded and displayed with size info
- **Type Identification** -- Shows the UTI (Uniform Type Identifier) for each entry

## Prerequisites

- **Rust** 1.77.2 or later -- [install via rustup](https://rustup.rs)
- **macOS** (clipboard reading uses NSPasteboard; other platforms return empty data)
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

The built `.app` bundle is output to:

```
src-tauri/target/release/bundle/macos/Clipboard Investigator.app
```

## Project Structure

```
clipboard-investigator/
  src-tauri/
    src/
      lib.rs          # Core logic: NSPasteboard reading, ClipboardEntry struct
      main.rs         # Bootstrap entry point
    Cargo.toml        # Rust dependencies
    tauri.conf.json   # Tauri app config (window, CSP, bundling)
    capabilities/     # Tauri permission definitions
    icons/            # App icons (.icns, .ico, .png)
  frontend/
    index.html        # HTML entry point
    main.js           # UI logic, event handlers, Tauri IPC calls
    style.css         # Application styles
  docs/
    index.html        # GitHub Pages website
```

## How It Works

The Rust backend uses the `objc` crate to call macOS NSPasteboard APIs via Objective-C interop. It iterates all registered clipboard types, classifies each by UTI, and extracts the data -- text is returned directly, binary data is base64-encoded.

The frontend receives a `Vec<ClipboardEntry>` (serialized as JSON) via Tauri's IPC bridge and renders it as an HTML table with type, content preview, and size columns.

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Desktop Framework | Tauri | 2.3.1 |
| Backend Language | Rust | 1.77.2+ |
| Frontend | Vanilla JS / HTML / CSS | -- |
| macOS Interop | `objc` crate | 0.2 |
| Serialization | `serde` / `serde_json` | 1.0 |
| Binary Encoding | `base64` crate | 0.22 |

## License

MIT

Based on [clipboard-inspector](https://github.com/evercoder/clipboard-inspector) by [Moqups Labs](https://labs.moqups.com).
