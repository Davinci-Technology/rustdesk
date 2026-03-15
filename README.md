# Compass RMM

A headless remote desktop agent built on [RustDesk](https://github.com/rustdesk/rustdesk), designed to be orchestrated by the Compass helpdesk agent (C#). It runs as a background service with no UI, accepts incoming remote desktop sessions, and can launch a viewer for outbound connections.

## Architecture

Compass RMM ships two binaries:

| Binary | Description |
|--------|-------------|
| `compass-rmm.exe` | Pure Rust, headless server + CLI orchestrator. No GUI. |
| `compass-viewer.exe` | Flutter runner (built from the standard RustDesk Flutter runner). Provides the remote desktop client UI. |

Both binaries link against `librustdesk.dll` (built with the `compass-rmm` feature), which sets the app name to **"Compass RMM"** and disables tray/service/install UI paths.

## Building

### Prerequisites

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust (stable) | 1.75+ | `rustup toolchain install stable` |
| Rust (nightly) | any | Required by `cargo-expand` during codegen. `rustup toolchain install nightly` |
| vcpkg | latest | With `libvpx`, `libyuv`, `opus`, `aom` installed. Set `VCPKG_ROOT` env var. |
| Flutter SDK | **3.24.5** | **Exact version required.** Newer versions have breaking Dart API changes. |
| LLVM/Clang | any | Required by `ffigen` during Flutter bridge codegen (install via system package manager). |
| cargo-expand | 1.0.x | `cargo install cargo-expand` (used by flutter_rust_bridge_codegen). |
| flutter_rust_bridge_codegen | **1.80.1** | **Must match** the `flutter_rust_bridge` crate version in Cargo.toml. `cargo install flutter_rust_bridge_codegen --version 1.80.1` |
| Visual Studio Build Tools (Windows) | 2022 | C++ desktop workload for Windows builds. |
| Developer Mode (Windows) | enabled | Required for Flutter plugin symlinks. |

### Installing Flutter 3.24.5

```bash
git clone https://github.com/flutter/flutter.git -b stable /opt/flutter --depth=1
cd /opt/flutter && git fetch --tags --depth=1 origin refs/tags/3.24.5 && git checkout 3.24.5
export PATH="/opt/flutter/bin:$PATH"
flutter --version  # Should show 3.24.5
```

### Build Steps (Full Pipeline)

The build has 4 phases. Steps 1-2 can run in parallel.

#### Step 0: Generate Flutter-Rust Bridge Bindings

This generates `src/bridge_generated.rs`, `src/bridge_generated.io.rs`, and `flutter/lib/generated_bridge.dart`. These files are **not checked into git** and must be regenerated when `src/flutter_ffi.rs` changes.

**Important:** The codegen runs `cargo expand` internally, which defaults to the binary target. You must temporarily disable bin targets so it expands the library instead:

```bash
# 1. Temporarily edit Cargo.toml:
#    - Comment out all [[bin]] sections
#    - Comment out `default-run = "rustdesk"`
#    - Add `autobins = false` under [package]
#    - Add "flutter" to the `default` features list

# 2. Run codegen
export VCPKG_ROOT=/path/to/vcpkg
flutter_rust_bridge_codegen \
  --rust-input ./src/flutter_ffi.rs \
  --dart-output ./flutter/lib/generated_bridge.dart \
  --rust-crate-dir . \
  --rust-output ./src/bridge_generated.rs \
  --skip-deps-check

# 3. Revert all Cargo.toml changes
```

If codegen succeeds, you should see:
- `src/bridge_generated.rs` — ~5000 lines
- `src/bridge_generated.io.rs` — ~2400 lines
- `flutter/lib/generated_bridge.dart` — ~14000 lines

If the Dart file is only ~366 lines with an empty `RustdeskImpl`, the codegen expanded the wrong target (binary instead of library).

#### Step 1: Build `compass-rmm.exe` (headless agent)

```bash
VCPKG_ROOT=/path/to/vcpkg cargo build --bin compass-rmm --features compass-rmm --release
# Output: target/release/compass-rmm(.exe)
```

This binary does not need Flutter or the bridge codegen.

#### Step 2: Build `librustdesk.dll` (shared core library)

```bash
VCPKG_ROOT=/path/to/vcpkg cargo build --lib --features compass-rmm,flutter --release
# Output: target/release/librustdesk.dll (Windows) or librustdesk.so (Linux)
```

Requires the bridge codegen output from Step 0.

#### Step 3: Build `compass-viewer.exe` (Flutter remote desktop viewer)

```bash
export COMPASS_VIEWER=1   # Controls binary name + exe branding
cd flutter
flutter pub get
flutter build windows --release   # or: flutter build linux --release
# Output: flutter/build/windows/x64/runner/Release/compass-viewer.exe
```

The `COMPASS_VIEWER` env var is read by `flutter/windows/CMakeLists.txt` to:
- Set the binary name to `compass-viewer` instead of `rustdesk`
- Pass a `COMPASS_VIEWER` preprocessor define to the C++ runner
- Gate the version info in `Runner.rc` (CompanyName, ProductName, etc.)

### Deployment Bundle

Copy all files from `flutter/build/windows/x64/runner/Release/` plus `compass-rmm.exe`:

```
compass-rmm.exe              # From target/release/
compass-viewer.exe            # Flutter runner binary
librustdesk.dll               # Shared Rust core (copied by Flutter build)
flutter_windows.dll           # Flutter engine
data/                         # Flutter assets
  flutter_assets/
  app.so
  icudtl.dat
*.dll                         # Plugin DLLs (desktop_drop, screen_retriever, etc.)
```

### Jenkins / CI Notes

A CI pipeline should run these steps in order:

```groovy
// Jenkinsfile pseudocode
stage('Setup') {
    // Install Rust stable + nightly, Flutter 3.24.5, vcpkg, LLVM
    // Install cargo tools: cargo-expand, flutter_rust_bridge_codegen@1.80.1
}
stage('Codegen') {
    // Temporarily patch Cargo.toml (disable bins, enable flutter default)
    // Run flutter_rust_bridge_codegen
    // Revert Cargo.toml
}
stage('Build Rust') {
    parallel {
        stage('compass-rmm')   { sh 'cargo build --bin compass-rmm --features compass-rmm --release' }
        stage('librustdesk.dll') { sh 'cargo build --lib --features compass-rmm,flutter --release' }
    }
}
stage('Build Flutter Viewer') {
    sh 'cd flutter && COMPASS_VIEWER=1 flutter pub get && COMPASS_VIEWER=1 flutter build windows --release'
}
stage('Package') {
    // Collect outputs into deployment bundle
}
```

### Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| `bridge_generated.rs` not found | Flutter bridge codegen not run | Run Step 0 |
| `EventToUI: IntoIntoDart<_>` not satisfied | bridge_generated.rs is empty/stale | Regenerate with codegen (ensure lib is expanded, not bin) |
| `generated_bridge.dart` has empty `RustdeskImpl` | `cargo expand` expanded binary instead of library | Comment out `[[bin]]` sections and `default-run` in Cargo.toml, add `autobins = false` |
| `DialogTheme`/`TabBarTheme` type errors | Wrong Flutter SDK version | Must use Flutter **3.24.5** exactly |
| `get_clients_state` defined multiple times | Feature flag conflict | The `compass-rmm` version is `get_clients_state_tuples()`, the `flutter` version is `get_clients_state()` |
| `VCPKG_ROOT` not found | Missing env var | Set `VCPKG_ROOT` pointing to your vcpkg installation |
| Symlink errors on Windows | Developer Mode not enabled | Run `start ms-settings:developers` and enable Developer Mode |

## Command-Line Usage

### `--headless` -- Run as a headless server

Starts the agent in headless mode: no tray, no UI, accepts incoming remote connections.

```
compass-rmm.exe --headless [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--rendezvous-server <addr>` | Set the rendezvous server address |
| `--relay-server <addr>` | Set the relay server address |
| `--key <key>` | Set the API/license key |
| `--password <pw>` | Set the permanent password |

**Startup output (JSON to stdout):**

```json
{"event":"started","id":"123456789","version":"1.4.1"}
```

**Session events (JSON to stdout):**

```json
{"event":"session_start","id":1,"peer_id":"987654321","name":"Tech-PC","authorized":true,"is_file_transfer":false,"keyboard":true,"clipboard":true,"audio":true}
{"event":"session_end","id":1,"close":true}
{"event":"message","id":1,"text":"Hello from remote"}
{"event":"file_transfer","action":"send","log":"file.txt"}
```

On Windows, incoming sessions also trigger a toast notification: *"Remote session from Tech-PC (987654321)"*.

### Launching the viewer

The C# agent is responsible for launching `compass-viewer.exe` directly:

```
compass-viewer.exe --connect <peer_id> --password <pw>
```

The `--connect` flag is in the Flutter runner's `parameters_white_list`, so multiple viewer instances can run simultaneously (connecting to different machines). The viewer loads `librustdesk.dll` which handles rendezvous/relay connection setup.

### Other flags

| Flag | Description |
|------|-------------|
| `--version` | Print version and exit |
| `--build-date` | Print build date and exit |
| `--get-id` | Print the agent's RustDesk ID and exit |
| `--set-id <id>` | Set the agent's ID (requires installation + admin) |
| `--password <pw>` | Set permanent password (requires installation + admin) |
| `--config <name>` | Apply an encrypted config string (requires installation + admin) |
| `--option <key> [value]` | Get or set a config option (requires installation + admin) |
| `--import-config <path>` | Import a `.toml` config file |
| `--server` | Start the server process (used internally) |

## IPC Protocol

The Compass C# agent communicates with `compass-rmm.exe` over a named pipe. Messages are serialized as JSON using the `Data::AgentQuery` / `Data::AgentResponse` envelope.

### Query Commands

Send a `Data::AgentQuery(AgentQueryType)` message. The agent replies with `Data::AgentResponse(AgentResponseData)`.

#### `GetId`

Returns the agent's RustDesk ID.

```
Query:  {"t": "GetId"}
Response: {"t": "Id", "c": "123456789"}
```

#### `GetPassword`

Returns the permanent password.

```
Query:  {"t": "GetPassword"}
Response: {"t": "Password", "c": "mysecretpw"}
```

#### `SetPassword`

Sets a new permanent password.

```
Query:  {"t": "SetPassword", "c": "newpassword"}
Response: {"t": "Ok"}
```

#### `GetConfig`

Returns all key configuration values.

```
Query:  {"t": "GetConfig"}
Response: {"t": "Config", "c": {"id": "123456789", "custom-rendezvous-server": "rs.example.com", "relay-server": "relay.example.com", "key": "abc..."}}
```

#### `SetConfig`

Sets a single config option.

```
Query:  {"t": "SetConfig", "c": {"key": "custom-rendezvous-server", "value": "rs.example.com"}}
Response: {"t": "Ok"}
```

#### `GetSessionList`

Returns all active remote sessions.

```
Query:  {"t": "GetSessionList"}
Response: {"t": "SessionList", "c": [{"id": 1, "peer_id": "987654321", "name": "Tech-PC", "authorized": true, "is_file_transfer": false}]}
```

Each session entry is an `AgentSessionInfo`:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i32` | Connection ID |
| `peer_id` | `String` | Remote peer's RustDesk ID |
| `name` | `String` | Remote peer's display name |
| `authorized` | `bool` | Whether the connection is authorized |
| `is_file_transfer` | `bool` | Whether this is a file transfer session |

#### `Shutdown`

Terminates the agent process.

```
Query:  {"t": "Shutdown"}
Response: (process exits)
```

### Error Responses

Any command can return an error:

```json
{"t": "Error", "c": "description of what went wrong"}
```

## Feature Flags

The `compass-rmm` Cargo feature controls all Compass-specific behavior:

- Sets `APP_NAME` to "Compass RMM"
- Sets Windows exe metadata (ProductName, FileDescription, OriginalFilename)
- Disables: tray icon, service install/uninstall, OS service mode, auto-update, installation UI
- Enables: `--headless` mode, IPC agent protocol, headless connection manager

## Based On

This is a fork of [RustDesk](https://github.com/rustdesk/rustdesk) -- an open-source remote desktop application.
