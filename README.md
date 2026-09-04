# System Analyzer

Cross-platform desktop system diagnostics app built with a **C++20 engine**, **Tauri 2** shell, and **Vue 3** frontend.

## What It Does

### Disk Analyzer ✅

QDirStat-style storage analyzer — scan any directory and get:

- Interactive treemap visualization
- Directory drill-down with breadcrumb navigation
- Largest files list
- File-type breakdown
- Mounted volume info
- Scan progress + cancellation

### Network Analyzer

Per-interface and per-application network monitoring (in progress):

- Real-time throughput charts
- 24-hour usage history (SQLite-backed)
- Per-application network attribution (Linux)
- Interface stats (RX/TX bytes, packets, errors, drops)

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Vue 3 frontend (apps/web)                                   │
│  Dashboard · Treemap · Breadcrumbs · Network panels          │
└────────────────────────────┬─────────────────────────────────┘
                             │ Tauri commands
┌────────────────────────────▼─────────────────────────────────┐
│  Tauri 2 desktop shell (apps/desktop/src-tauri)              │
│  Sidecar orchestration · Progress events · SQLite storage    │
└────────────────────────────┬─────────────────────────────────┘
                             │ stdout → JSON / stderr → PROGRESS
┌────────────────────────────▼─────────────────────────────────┐
│  C++20 scanning engine (cpp/)                                │
│  Filesystem traversal · Network parsing · JSON serialization │
└──────────────────────────────────────────────────────────────┘
```

The C++ engine is a standalone CLI binary — no dependency on Tauri or Vue. Build, test, and run it independently.

### Platform Abstraction

Platform-specific code lives behind C++ interfaces with per-OS factories:

```
cpp/src/platform/
├── common/     portable scanner/mapper (shared)
├── linux/      LinuxFileScanner, LinuxDiskUsageProvider, LinuxVolumeProvider
├── windows/    Windows* providers (Win32 APIs)
└── macos/      MacOS* providers (statvfs, getmntinfo)
```

Application code never branches on the OS.

## Current Status

| Module | Linux | Windows | macOS |
| ------- | ------- | --------- | ------- |
| Disk Analyzer | Full | Platform layer done | Platform layer done |
| Network Analyzer | Core + UI | Planned | Planned |

**Disk Analyzer v1** is the first production milestone — cross-platform bundling verified via CI (.deb, .AppImage, NSIS .exe).

**Network Analyzer** has its Linux implementation, telemetry, database storage, and Vue UI components in place. Windows and macOS platform layers are pending.

## Build & Run

### Prerequisites

- C++20 compiler (GCC 13+, MSVC 19+, Clang 17+)
- CMake 3.20+ + Ninja
- Rust 1.77+ (for Tauri)
- Node.js 22+ + pnpm

### Quick Start

```bash
# 1. Build the C++ engine
./scripts/build-engine.sh          # Linux/macOS
.\scripts\build-engine.ps1        # Windows

# 2. Install frontend deps
cd apps/web && pnpm install

# 3. Run the full desktop app
pnpm tauri:dev
```

### Run Engine Standalone

```bash
./build/system-analyzer-engine /path/to/scan
# Outputs JSON ScanResult to stdout, progress to stderr
```

### Run Tests

```bash
cd build && ctest                 # C++ tests
cd apps/web && pnpm test:unit     # Vue tests
cd apps/desktop/src-tauri && cargo test  # Rust tests
```

## Project Structure

```
system-analyzer/
├── cpp/                    # C++20 engine
│   ├── include/            # Interfaces & domain models
│   ├── src/                # Implementations + platform providers
│   └── tests/              # Unit + contract tests
├── apps/
│   ├── web/                # Vue 3 + TypeScript frontend
│   └── desktop/
│       └── src-tauri/      # Tauri 2 shell (Rust)
├── scripts/                # Build scripts + version sync
├── docs/                   # Development logs, cheatsheets
├── CMakeLists.txt          # Top-level CMake (engine + tests)
└── VERSION                 # Single source of truth (0.1.0)
```

## Key Design Decisions

- **Stable JSON contract** between engine and UI — schema tested by contract tests on every platform
- **Thin Tauri layer** — desktop integration only, no core logic
- **Portable domain layer** — core analyzer has zero OS-specific code
- **Native APIs where needed** — `std::filesystem` preferred, OS APIs for platform-specific data

## Roadmap

- [x] Disk Analyzer core (scan, treemap, breakdowns)
- [x] Cross-platform packaging (Linux/Windows CI)
- [x] Network telemetry + SQLite storage (Linux)
- [x] Network UI components (Vue)
- [ ] Network Analyzer Windows/macOS platform layers
- [ ] Disk Analyzer v1 stable release
- [ ] File search, duplicate detection, cleanup tools

## Documentation

- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) — Build instructions, packaging status, release runbook
- [`docs/CPP-CHEATSHEET.md`](docs/CPP-CHEATSHEET.md) — CMake, Ninja, ctest, GDB commands
- [`docs/VUE-CHEATSHEET.md`](docs/VUE-CHEATSHEET.md) — Vue, TypeScript, Vite notes

## License

MIT
