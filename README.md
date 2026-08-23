# System Analyzer

System Analyzer is a cross-platform desktop system diagnostics application
built around a native C++20 engine, a Tauri 2 desktop shell, and a Vue 3
frontend.

The first module is **Disk Analyzer**, a QDirStat/WinDirStat-style storage
analyzer that scans directories and provides:

- Interactive storage treemaps
- Directory drill-down and breadcrumb navigation
- Largest-file analysis
- File-type breakdowns
- Mounted-volume information
- Scan progress and cancellation
- Cross-platform filesystem analysis

The C++ engine is intentionally independent of the desktop UI. It runs as a
standalone CLI and communicates with the Tauri shell through a stable JSON
contract.

The second major module will be **Network Analyzer**, which will reuse the
same C++/Tauri/Vue architecture to provide network throughput, aggregate
24-hour usage, and per-application network consumption across supported
operating systems.

---

## Architecture

System Analyzer is divided into three primary layers, each with a focused
responsibility:

```text
┌──────────────────────────────────────────────────────────────┐
│  Vue 3 frontend (apps/web)                                   │
│                                                              │
│  Dashboard · Treemap · Breadcrumbs · Largest Files           │
│  File Types · Volumes · Storage Usage · Scan Controls        │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             │ Tauri commands
                             │
                             │ scan_directory
                             │ cancel_scan
                             │ reveal_in_file_manager
                             │
┌────────────────────────────▼─────────────────────────────────┐
│  Tauri 2 desktop shell                                       │
│  apps/desktop/src-tauri                                      │
│                                                              │
│  Sidecar orchestration · Progress events · Cancellation      │
│  OS integration · File-manager integration                   │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             │ stdout → JSON ScanResult
                             │ stderr → PROGRESS:<n>
                             │
┌────────────────────────────▼─────────────────────────────────┐
│  C++20 scanning engine                                       │
│  cpp/                                                        │
│                                                              │
│  Filesystem traversal · Directory aggregation                │
│  File analysis · Volume information · JSON serialization     │
└──────────────────────────────────────────────────────────────┘
````

The engine is a plain CLI binary:

```text
system-analyzer <directory>
```

It has no dependency on Tauri, Vue, or the desktop application and can be
built, tested, debugged, and executed independently.

### Platform abstraction

Platform-specific functionality is isolated behind C++ interfaces and
factories. Application-level code does not contain OS-specific implementation
logic.

```text
cpp/src/platform/
├── common/
│   └── portable scanner/mapper
│
├── linux/
│   ├── LinuxFileScanner
│   ├── LinuxDiskUsageProvider
│   └── LinuxVolumeProvider
│
├── windows/
│   ├── WindowsFileScanner
│   ├── WindowsDiskUsageProvider
│   └── WindowsVolumeProvider
│
└── macos/
    ├── MacOSFileScanner
    ├── MacOSDiskUsageProvider
    └── MacOSVolumeProvider
```

The application layer requests providers through the platform factory:

```cpp
auto volumeProvider = createVolumeProvider();
auto diskUsageProvider = createDiskUsageProvider();
auto fileScanner = createFileScanner();
```

Platform selection is handled at the platform/build boundary rather than
through OS-specific branching throughout the application.

The same approach will be used for Network Analyzer:

```text
NetworkUsageProvider
├── LinuxNetworkUsageProvider
├── WindowsNetworkUsageProvider
└── MacOSNetworkUsageProvider
```

This keeps the domain model and JSON contract consistent while allowing each
operating system to use its native APIs.

### Cross-platform JSON contract

Every platform produces the same `ScanResult` structure.

The actual values naturally differ between machines, but the following must
remain stable across platforms:

- Field names
- Field types
- Enum domains
- Error semantics
- Volume semantics
- Read-only semantics
- Serialization behavior

The `scan-schema-contract` test performs a real application-level scan and
validates the serialized contract. This prevents Linux, Windows, and macOS
sidecars from drifting apart and requiring platform-specific frontend logic.

---

## Tech Stack

| Layer           | Technology                         |
| --------------- | ---------------------------------- |
| Engine          | C++20, CMake 3.20+, Ninja          |
| JSON            | nlohmann/json                      |
| Desktop shell   | Rust, Tauri 2                      |
| Frontend        | Vue 3, Composition API, TypeScript |
| Build tooling   | Vite, pnpm                         |
| Testing         | ctest, Vitest                      |
| Static analysis | ESLint, Oxlint                     |
| Formatting      | Prettier                           |
| CI/CD           | GitHub Actions                     |

The frontend intentionally uses Vue composables rather than introducing a
global state-management library for the current scope.

---

## Repository Layout

```text
.
├── cpp/
│   ├── include/
│   │   └── system_analyzer/
│   │       ├── domain/
│   │       ├── core/
│   │       └── platform/
│   │
│   ├── src/
│   │   ├── app/
│   │   ├── core/
│   │   ├── platform/
│   │   │   ├── common/
│   │   │   ├── linux/
│   │   │   ├── windows/
│   │   │   └── macos/
│   │   └── serialization/
│   │
│   ├── tests/
│   │   └── ctest unit and contract tests
│   │
│   └── third_party/
│       └── nlohmann/json
│
├── apps/
│   ├── desktop/
│   │   └── src-tauri/
│   │       ├── src/
│   │       │   ├── commands/
│   │       │   └── platform/
│   │       └── binaries/
│   │           └── staged sidecars
│   │
│   └── web/
│       └── src/
│           ├── components/
│           ├── composables/
│           ├── services/
│           └── types/
│
├── scripts/
│   ├── build-engine.sh
│   ├── build-engine.ps1
│   └── sync-release-version.mjs
│
├── docs/
│   ├── DEVELOPMENT.md
│   ├── CPP-CHEATSHEET.md
│   └── VUE-CHEATSHEET.md
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
└── VERSION
```

The generated sidecar binaries under
`apps/desktop/src-tauri/binaries/` are gitignored and must be rebuilt for the
target operating system and architecture.

---

## Getting Started

### Prerequisites

#### All platforms

- CMake >= 3.20
- Ninja
- C++20 compiler
- Rust stable
- Tauri CLI
- Node.js `^22.18.0 || >=24.12.0`
- pnpm

#### Linux

Install the Tauri/WebKit dependencies required by the desktop shell:

```bash
sudo apt update

sudo apt install -y \
  build-essential \
  cmake \
  ninja-build \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Refer to the official Tauri documentation for any additional distribution
specific requirements.

#### Windows

- Visual Studio Build Tools or Visual Studio with the MSVC C++ workload
- Windows SDK
- CMake
- Ninja
- Rust stable
- WebView2

#### macOS

macOS support is architecture-ready and tested in CI, but release packaging
is currently deferred.

---

## Build the C++ Engine

### Linux / macOS

```bash
./scripts/build-engine.sh
```

### Windows

```powershell
.\scripts\build-engine.ps1
```

The build scripts:

1. Configure CMake.
2. Build the C++ engine.
3. Build the C++ test suite.
4. Run `ctest`.
5. Stage the sidecar binary under
   `apps/desktop/src-tauri/binaries/`.
6. Use the correct Tauri target triple for the current platform.

For example:

```text
Linux:
system-analyzer-x86_64-unknown-linux-gnu

Windows:
system-analyzer-x86_64-pc-windows-msvc

macOS:
system-analyzer-<target-triple>
```

Useful environment variables:

```text
ENGINE_GENERATOR
ENGINE_BUILD_TYPE
```

Defaults:

```text
ENGINE_GENERATOR=Ninja
ENGINE_BUILD_TYPE=Debug
```

### Run the engine directly

After building:

```bash
./build/system-analyzer /path/to/directory
```

The engine writes the serialized `ScanResult` to stdout.

Progress events are written to stderr:

```text
PROGRESS:1
PROGRESS:2
PROGRESS:3
...
```

This separation allows the Tauri shell to consume structured scan results
without mixing progress messages into the JSON stream.

---

## Run the Desktop Application

From the frontend workspace:

```bash
cd apps/web
pnpm install
pnpm tauri:dev
```

The development application consists of:

```text
Vite
  ↓
Vue 3
  ↓
Tauri 2
  ↓
C++ sidecar
```

The sidecar is a compiled binary and is not hot-reloaded.

After modifying C++ code, rebuild the engine:

```bash
./scripts/build-engine.sh
```

or on Windows:

```powershell
.\scripts\build-engine.ps1
```

Then restart the Tauri development application if necessary.

---

## Web-only Development

For frontend-only development:

```bash
cd apps/web
pnpm dev
```

This is useful for working on dashboard components and visualizations without
starting the native Tauri shell.

Native functionality such as the C++ sidecar, directory selection, progress
events, and file-manager integration requires the Tauri application.

---

## Scan Architecture

A normal scan follows this pipeline:

```text
User selects directory
        │
        ▼
Vue ScanControls
        │
        ▼
useScanner()
        │
        ▼
Tauri scan_directory command
        │
        ▼
C++ sidecar
        │
        ├── filesystem traversal
        ├── directory aggregation
        ├── file-type aggregation
        ├── largest-file calculation
        └── volume/disk information
        │
        ▼
ScanResult JSON
        │
        ▼
Tauri
        │
        ▼
Vue dashboard
```

During scanning:

```text
C++ stderr
    │
    └── PROGRESS:<n>
            │
            ▼
       Tauri event
            │
            ▼
       Vue progress UI
```

This keeps the actual scan computation out of the frontend and avoids
duplicating filesystem logic between Vue, Rust, and C++.

---

## Directory Drill-down

The treemap supports directory navigation.

For example:

```text
/home/user
     │
     ├── Documents
     ├── Downloads
     ├── Projects
     └── Videos
            │
            │ click
            ▼
/home/user/Videos
     │
     ├── Movies
     ├── Recordings
     └── Tutorials
```

Each drill-down starts a new scan rooted at the selected directory.

The complete dashboard is therefore scoped to the current location:

- Summary
- Treemap
- Largest files
- File types
- Volumes
- Disk usage

Breadcrumb navigation allows returning to any previously visited ancestor.

The C++ engine does not need to understand UI navigation. It simply scans the
directory it receives.

---

## Testing

### C++ Engine

```bash
ctest --output-on-failure --test-dir build
```

The C++ test suite covers:

- Filesystem scanning
- Directory aggregation
- Disk usage providers
- Volume providers
- Serialization
- Error behavior
- Scan result contracts
- Platform-specific providers

### Rust/Tauri

```bash
cd apps/desktop/src-tauri

cargo check
cargo test
```

### Frontend

```bash
cd apps/web

pnpm lint
pnpm type-check
pnpm test:unit
pnpm build-only
```

### Cross-platform contract

The `scan-schema-contract` test is particularly important.

Every supported platform must produce a compatible `ScanResult`:

```text
Linux ───────┐
Windows ─────┼──→ stable ScanResult contract → Vue
macOS ───────┘
```

Platform-specific implementation differences must not leak into the frontend
API.

---

## Continuous Integration

CI builds and tests the project natively on the target operating systems.

The v1 release targets are:

```text
Linux x86_64
Windows x86_64
```

macOS is architecture-ready and can be included in the CI validation matrix,
but release bundling remains deferred until the macOS platform layer and
release requirements are finalized.

CI performs:

```text
C++ configure
      ↓
C++ build
      ↓
ctest
      ↓
Rust check/test
      ↓
Frontend lint/type-check/tests/build
      ↓
Tauri validation
```

There is no C++ cross-compilation in the normal release pipeline.

Native builds are preferred because they provide:

- Native compiler behavior
- Native system headers
- Native platform APIs
- More reliable filesystem tests
- Easier debugging
- Fewer cross-compilation toolchain issues

CI caching should be enabled for:

- Rust dependencies
- pnpm store
- CMake builds/ccache where appropriate

---

## Cross-platform Strategy

The project deliberately separates portable application logic from
OS-specific implementations.

### Linux

Current and planned native APIs include:

```text
Filesystem
├── std::filesystem
├── statvfs
└── /proc and /sys where required

Volumes
├── mount information
└── filesystem metadata
```

### Windows

Native providers use Windows APIs where required:

```text
Volumes
├── GetLogicalDrives
├── GetDiskFreeSpaceExW
├── GetVolumeInformationW
└── volume flags

Filesystem
└── std::filesystem
```

### macOS

The platform layer is designed around native macOS/POSIX facilities:

```text
Filesystem
└── std::filesystem

Disk usage
└── statvfs

Volumes
└── getmntinfo
```

The goal is not to make the OS implementations identical internally.

The goal is to expose **identical domain-level behavior** to the rest of the
application.

---

## Sidecar Packaging

Tauri's `externalBin` mechanism is used to package the C++ engine alongside
the desktop application.

The sidecar must be built for the target OS and architecture.

```text
Linux
└── system-analyzer-x86_64-unknown-linux-gnu

Windows
└── system-analyzer-x86_64-pc-windows-msvc

macOS
└── platform-specific target triple
```

The platform-specific binary is selected by Tauri during application
packaging.

Sidecars are not committed to the repository.

---

## Versioning

`VERSION` at the repository root is the single source of truth.

For example:

```text
0.2.0
```

After changing the version:

```bash
node scripts/sync-release-version.mjs
```

The synchronization script propagates the version into:

```text
apps/web/package.json
apps/desktop/src-tauri/tauri.conf.json
apps/desktop/src-tauri/Cargo.toml
CMakeLists.txt
```

This prevents independent package versions from drifting apart.

---

## Downloads

Stable releases are published on the [GitHub Releases](https://github.com/Temkum/storage-analyzer/releases) page.
Download the asset for your platform and architecture from the latest tagged
release (for example, `v0.2.0`).

### Platform support matrix

| Platform | Architecture | Package format | Status |
| --- | --- | --- | --- |
| Linux | x86_64 | `.deb`, `.AppImage` | ✅ v1 |
| Windows | x86_64 | `.msi` (NSIS) | ✅ v1 |
| macOS | x86_64 / arm64 | `.dmg`, `.app` (unsigned) | ⏳ planned (v1 architecture-ready) |

macOS binaries are architecture-ready and exercised in CI, but GUI bundling is
deferred to a later release. macOS users can build from source (see
[Run the Desktop Application](#run-the-desktop-application) to run Disk Analyzer locally.

### Installation by platform

#### Linux

**Debian / Ubuntu (.deb):**

```bash
sudo apt install ./system-analyzer-x86_64-unknown-linux-gnu.deb
```

**AppImage (portable, no root required):**

```bash
chmod +x SystemAnalyzer-x86_64.AppImage
./SystemAnalyzer-x86_64.AppImage
```

> Some distributions (notably Arch-based) require the `fuse2` or `fuse3`
> compatibility package to execute AppImage files:
>
> ```bash
> sudo pacman -S fuse2    # or fuse3
> ```

#### Windows

Run the installer and follow the setup wizard:

```text
SystemAnalyzer-0.2.0-x86_64.msi
```

> The v1 release is **not code-signed**. Windows SmartScreen may display an
> "Unknown publisher" warning. Click **More info** → **Run anyway** to proceed.
> Code signing will be added once production signing requirements are defined.

#### macOS (from source, until signed bundling ships)

macOS GUI packaging is deferred for v1, but the engine builds and runs natively.
See [Run the Desktop Application](#run-the-desktop-application) for `pnpm tauri:dev`
instructions to build and run locally.

---

## Releases (maintainer section)

The v1 release targets are:

### Linux

```text
.deb
AppImage
```

### Windows

```text
NSIS installer
```

### macOS

Release packaging is deferred for v1.

The intended future macOS targets are:

```text
Apple Silicon
Intel
```

where supported by the final release configuration.

### Release process

1. Update `VERSION`.
2. Synchronize package versions:

```bash
node scripts/sync-release-version.mjs
```

1. Commit the version change.
2. Create a release tag:

```bash
git tag v0.2.0
git push origin v0.2.0
```

1. GitHub Actions builds the native engine and desktop application.
2. Release artifacts are uploaded.
3. A GitHub Release is drafted with the generated artifacts and changelog.

---

## Known Release Caveats

### Windows

The current release configuration does not include code signing.

Users may therefore see Windows SmartScreen warnings when launching the
installer or application.

Code signing will be addressed once production distribution requirements are
defined.

### Linux AppImage

Some Linux distributions require FUSE support to execute AppImage files.

Users on affected distributions may need to install the appropriate FUSE
compatibility package.

### macOS

macOS is not currently part of the v1 release artifact set.

Code signing, notarization, and distribution requirements will be addressed
when macOS release support is enabled.

### Auto-updates

Automatic application updates are currently out of scope.

---

## Roadmap

### Disk Analyzer v1

- [x] Native C++ scanning engine
- [x] Directory size aggregation
- [x] Scan result JSON contract
- [x] Vue 3 dashboard
- [x] Interactive treemap
- [x] Directory drill-down
- [x] Breadcrumb navigation
- [x] Largest-file analysis
- [x] File-type analysis
- [x] Volume analysis
- [x] Scan progress
- [ ] Scan cancellation
- [ ] Permission/error handling hardening
- [ ] Symlink edge-case handling
- [ ] Disappearing-file handling
- [ ] Large-tree performance pass
- [ ] Linux x86_64 packaging
- [ ] Windows x86_64 packaging
- [ ] macOS platform completion
- [ ] Production release validation

### System Analyzer Phase 2: Network Analyzer

Network Analyzer will reuse the same architecture:

```text
Vue 3
  ↓
Tauri 2
  ↓
C++ Network Analyzer
  ↓
Platform NetworkUsageProvider
```

Planned capabilities:

- [ ] Network interface discovery
- [ ] Current download throughput
- [ ] Current upload throughput
- [ ] Aggregate network usage
- [ ] 24-hour network usage history
- [ ] Per-application network consumption
- [ ] Linux network provider
- [ ] Windows network provider
- [ ] macOS network provider
- [ ] Cross-platform network JSON contract
- [ ] Network usage dashboard
- [ ] Application-level network breakdown

Per-application network accounting will use OS-specific implementations behind
the same C++ abstraction because Linux, Windows, and macOS expose different
network/process accounting mechanisms.

### Future Disk Analyzer Features

The following are intentionally deferred until the core analyzer is stable:

- [ ] File search
- [ ] Duplicate detection
- [ ] Similar-file detection
- [ ] Cleanup recommendations
- [ ] Delete/move operations
- [ ] Exclusion rules
- [ ] Historical scan comparison
- [ ] Scan caching
- [ ] Background monitoring

---

## Design Principles

### Platform independence at the domain layer

The core analyzer should not know whether it is running on Linux, Windows, or
macOS.

Platform-specific behavior belongs behind interfaces.

### Native APIs where they matter

Portable C++ facilities such as `std::filesystem` are preferred where they
provide sufficient functionality.

Native OS APIs are used when platform-specific information is required.

### Stable contracts

The Vue frontend should consume a stable domain model rather than platform
specific data structures.

### Thin Tauri layer

Tauri is the desktop integration layer, not the location of the core analyzer
logic.

The C++ engine remains independently executable and testable.

### Incremental complexity

Features are added to the platform abstraction only when the product actually
needs them.

This prevents premature platform-specific complexity from spreading through
the codebase.

---

## Development Workflow

A typical feature follows this path:

```text
1. Define domain model/interface
             ↓
2. Implement platform-independent behavior
             ↓
3. Implement platform providers
             ↓
4. Add C++ tests
             ↓
5. Validate JSON contract
             ↓
6. Expose through Tauri
             ↓
7. Add Vue composable/service
             ↓
8. Build UI
             ↓
9. Test native platforms
             ↓
10. Update documentation
```

For platform-specific functionality:

```text
Interface
    ↓
Factory
    ↓
Linux implementation
Windows implementation
macOS implementation
    ↓
Common domain model
    ↓
Stable JSON
    ↓
Vue
```

This structure is particularly important for Network Analyzer, where
per-application network accounting will require substantially different native
implementations across operating systems.

---

## Documentation

Additional project documentation:

- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)
  Development history, architecture decisions, build instructions, packaging
  status, and release sequencing.

- [`docs/CPP-CHEATSHEET.md`](docs/CPP-CHEATSHEET.md)
  Common CMake, Ninja, ctest, and GDB commands.

- [`docs/VUE-CHEATSHEET.md`](docs/VUE-CHEATSHEET.md)
  Vue, TypeScript, Vite, and frontend development notes.

---

## Project Status

System Analyzer is currently in active development.

**Disk Analyzer v1** is the first production milestone. The immediate focus is
to complete cross-platform support, harden scanning behavior, validate
performance, and produce reliable Linux and Windows desktop bundles.

Once Disk Analyzer v1 is stable, development will move to **Network Analyzer**,
which will extend the same C++20, Tauri 2, and Vue 3 architecture into
cross-platform network monitoring.

```
```
