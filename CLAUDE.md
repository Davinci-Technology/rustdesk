# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Compass RMM Build Commands

This fork produces two binaries (`compass-rmm` and `compass-viewer`) plus a shared DLL. See README.md for the full pipeline and troubleshooting guide.

### Quick Reference

```bash
# Headless agent (no Flutter dependency)
VCPKG_ROOT=/path/to/vcpkg cargo build --bin compass-rmm --features compass-rmm --release

# Shared DLL (needs flutter_rust_bridge codegen output)
VCPKG_ROOT=/path/to/vcpkg cargo build --lib --features compass-rmm,flutter --release

# Flutter viewer (needs COMPASS_VIEWER=1 for branding + binary name)
COMPASS_VIEWER=1 flutter pub get && COMPASS_VIEWER=1 flutter build windows --release
```

### Flutter Rust Bridge Codegen

The `src/bridge_generated.rs` and `flutter/lib/generated_bridge.dart` files are NOT in git. They must be regenerated when `src/flutter_ffi.rs` changes. The codegen tool is `flutter_rust_bridge_codegen` version **1.80.1** (must match the crate version in Cargo.toml).

**Critical gotcha:** The codegen runs `cargo expand` without `--lib`, so it expands the binary target by default and produces empty bindings. To fix this, you must temporarily:
1. Comment out all `[[bin]]` sections and `default-run` in Cargo.toml
2. Add `autobins = false` to `[package]`
3. Add `"flutter"` to the `default` features list
4. Run the codegen
5. Revert all Cargo.toml changes

A successful codegen produces ~5000-line `bridge_generated.rs` and ~14000-line `generated_bridge.dart`. If the Dart file is only ~366 lines, the codegen expanded the wrong target.

### Version Constraints

- **Flutter SDK must be 3.24.5** (set in `.github/workflows/flutter-build.yml`). Newer versions break Dart APIs.
- **flutter_rust_bridge_codegen must be 1.80.1** to match the crate. Version 1.82.6 has a parse error on the expanded output.
- **cargo-expand** needs the nightly Rust toolchain installed.

### Original RustDesk Build Commands (not used for Compass)
- `python3 build.py --flutter` - Build standard RustDesk Flutter version
- `cargo run` - Build and run standard RustDesk (requires libsciter)
- `cd flutter && flutter build android` - Build Android APK
- `cd flutter && flutter test` - Run Flutter tests

## Project Architecture

### Directory Structure
- **`src/`** - Main Rust application code
  - `src/ui/` - Legacy Sciter UI (deprecated, use Flutter instead)
  - `src/server/` - Audio/clipboard/input/video services and network connections
  - `src/client.rs` - Peer connection handling
  - `src/platform/` - Platform-specific code
- **`flutter/`** - Flutter UI code for desktop and mobile
- **`libs/`** - Core libraries
  - `libs/hbb_common/` - Video codec, config, network wrapper, protobuf, file transfer utilities
  - `libs/scrap/` - Screen capture functionality
  - `libs/enigo/` - Platform-specific keyboard/mouse control
  - `libs/clipboard/` - Cross-platform clipboard implementation

### Key Components
- **Remote Desktop Protocol**: Custom protocol implemented in `src/rendezvous_mediator.rs` for communicating with rustdesk-server
- **Screen Capture**: Platform-specific screen capture in `libs/scrap/`
- **Input Handling**: Cross-platform input simulation in `libs/enigo/`
- **Audio/Video Services**: Real-time audio/video streaming in `src/server/`
- **File Transfer**: Secure file transfer implementation in `libs/hbb_common/`

### UI Architecture
- **Legacy UI**: Sciter-based (deprecated) - files in `src/ui/`
- **Modern UI**: Flutter-based - files in `flutter/`
  - Desktop: `flutter/lib/desktop/`
  - Mobile: `flutter/lib/mobile/`
  - Shared: `flutter/lib/common/` and `flutter/lib/models/`

## Important Build Notes

### Dependencies
- Requires vcpkg for C++ dependencies: `libvpx`, `libyuv`, `opus`, `aom`
- Set `VCPKG_ROOT` environment variable
- Download appropriate Sciter library for legacy UI support

### Ignore Patterns
When working with files, ignore these directories:
- `target/` - Rust build artifacts
- `flutter/build/` - Flutter build output
- `flutter/.dart_tool/` - Flutter tooling files

### Cross-Platform Considerations
- Windows builds require additional DLLs and virtual display drivers
- macOS builds need proper signing and notarization for distribution
- Linux builds support multiple package formats (deb, rpm, AppImage)
- Mobile builds require platform-specific toolchains (Android SDK, Xcode)

### Feature Flags
- `compass-rmm` - Compass RMM mode: sets APP_NAME to "Compass RMM", disables tray/service/install, enables headless mode and IPC agent protocol
- `flutter` - Enable Flutter UI and flutter_rust_bridge FFI bindings (requires codegen)
- `hwcodec` - Hardware video encoding/decoding
- `vram` - VRAM optimization (Windows only)
- `unix-file-copy-paste` - Unix file clipboard support
- `screencapturekit` - macOS ScreenCaptureKit (macOS only)

### Compass RMM Feature Combinations
- `compass-rmm` alone: builds `compass-rmm` binary (headless agent, no Flutter)
- `compass-rmm,flutter`: builds `librustdesk.dll` (shared core with Flutter FFI for the viewer)
- `flutter` alone: standard RustDesk Flutter build

### Compass Viewer CMake Flags
The `COMPASS_VIEWER` environment variable controls the Flutter Windows runner:
- `flutter/windows/CMakeLists.txt` reads it to set binary name to `compass-viewer`
- `flutter/windows/runner/CMakeLists.txt` passes it as a preprocessor define
- `flutter/windows/runner/Runner.rc` uses `#ifdef COMPASS_VIEWER` for exe metadata branding
- `flutter/windows/runner/main.cpp` has `--connect` in `parameters_white_list` for multi-instance support

### Config
All configurations or options are under `libs/hbb_common/src/config.rs` file, 4 types:
- Settings
- Local
- Display
- Built-in
