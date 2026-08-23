# Development Log

## Cross-Platform Packaging (Disk Analyzer v1)

Status: Implemented (Linux/Windows bundling verified via CI)

### Target matrix

| OS | Architecture | Bundles | Notes |
|----|--------------|---------|-------|
| Linux | x86_64 | `.deb`, `.AppImage` | release artifacts |
| Windows | x86_64 | NSIS `.exe` installer | unsigned (SmartScreen warning expected) |
| macOS | x86_64 | none yet | platform layer implemented, bundling deferred |

### Platform architecture

The C++ engine selects its providers through a factory
(`cpp/include/system_analyzer/platform/factory.hpp`), implemented once per
OS under `cpp/src/platform/<os>/PlatformFactory.cpp`. Application code never
branches on the operating system.

```text
cpp/src/platform/
├── common/    portable scanner/mapper (header-only, shared by all OSes)
├── linux/     LinuxFileScanner, LinuxDiskUsageProvider, LinuxVolumeProvider
├── windows/   Windows* providers (Win32: drives, GetDiskFreeSpaceExW)
└── macos/     MacOS* providers (statvfs, getmntinfo)
```

Rust-side OS integration lives in `apps/desktop/src-tauri/src/platform/`
(file-manager reveal per OS).

### Cross-platform contract test

`scan-schema-contract` (ctest) runs a real scan through the Application and
asserts the serialized `ScanResult` keeps a stable JSON schema — fields,
types, enum domain, error/volume semantics — on every platform. This is the
gate that keeps all sidecars wire-compatible with the Vue UI.

### Building

```bash
# Linux / macOS
./scripts/build-engine.sh

# Windows (PowerShell)
.\scripts\build-engine.ps1
```

Both scripts configure CMake, build the engine, run ctest, and stage the
sidecar into `apps/desktop/src-tauri/binaries/` with the correct target
triple (e.g. `system-analyzer-x86_64-unknown-linux-gnu`).

### Versioning

`VERSION` at the repo root is the single source of truth. After changing it:

```bash
node scripts/sync-version.mjs
```

This updates `apps/web/package.json`, `apps/desktop/src-tauri/tauri.conf.json`,
`Cargo.toml`, and `CMakeLists.txt`.

### CI

- `.github/workflows/ci.yml` — every push/PR: engine build + ctest, cargo
  check/test, web lint/type-check/tests/build on `ubuntu-latest` and
  `windows-latest` (native GCC/MSVC builds; no cross-compilation).
- `.github/workflows/release.yml` — on `v*` tags: builds engine, syncs the
  tag version, bundles `.deb` + `.AppImage` (Linux) and NSIS `.exe`
  (Windows), then drafts a GitHub release with the artifacts.

### Release runbook

1. Update `VERSION` (e.g. `0.1.0` → `0.2.0`), run
   `node scripts/sync-version.mjs`, commit.
2. Tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. CI builds both platforms and drafts the release; review and publish.

### Known caveats

- Windows binaries are unsigned; SmartScreen will warn on first run.
- AppImage requires FUSE (`libfuse2`) on some distributions.
- macOS bundling is deferred; the platform layer is already in place.

---

## Phase 0: Environment Setup

Status: Complete

### Frontend

- Node.js 25.2.0
- pnpm 11.7.0
- Vue 3
- TypeScript
- Vite 8.2.1
- Vue Router
- Pinia
- Vitest
- ESLint
- Oxlint
- Prettier

### C++ Toolchain

- GCC/G++ 13.3.0
- CMake 3.28.3
- Ninja 1.11.1
- GDB 15.1

### Validation

- Vue production build: passed
- Type checking: passed
- Unit tests: 1 passed
- Oxlint: 0 warnings, 0 errors
- C++ compilation: passed
- C++ executable: passed

---

## Phase 1: C++ Foundation

Status: In Progress

### Completed

- Created C++ source directory
- Created C++ entry point
- Configured CMake
- Selected C++20
- Configured Ninja generator
- Successfully compiled executable
- Successfully executed C++ application

### Current executable

`build/system-analyzer`

### Next

- Establish C++ project conventions
- Add headers and source separation
- Introduce core domain model
- Establish testing structure
- Begin filesystem abstraction

## Phase 1: C++ Foundation

Status: Complete

### Completed

- Created C++ source structure
- Configured CMake
- Configured Ninja
- Selected C++20
- Created first domain class
- Separated header and implementation
- Successfully compiled and linked multiple translation units
- Configured Debug build
- Debugged executable using GDB
- Set breakpoints
- Stepped through source code
- Inspected local variables

### Next

- Introduce filesystem domain model
- Define platform abstraction
- Implement Linux filesystem scanner
- Establish unit testing strategy

````

Then check:

```bash
git diff
```

and commit:

```bash
git add docs/ CMakeLists.txt cpp/
git commit -m "feat: establish C++ project foundation"
```

### Then we get to the real project

The next thing I want to build is **not yet the full disk scanner**.

We'll first define what a scan actually returns:

```text
Scan
 ├── path
 ├── type
 ├── size
 ├── modified time
 ├── file count
 └── children
```

Then we'll decide which pieces belong in the **platform-independent core** and which belong in the **Linux implementation**.

That is where you'll start learning the C++ architecture that will eventually allow us to run the same application on **Ubuntu, Windows, and macOS**.

---
### Recommended sequence

1. **File-level exploration**

   * Largest files per directory
   * File metadata
   * Reveal/open file
   * Graceful handling of deleted/inaccessible files

2. **File-type analysis**

   * Group by extension
   * Size and count per type
   * Percentage of scanned storage
   * Better visualization
   * Examples: `.mp4`, `.zip`, `.jpg`, `.js`, `.pdf`

3. **Volume/storage analysis**

   * Mounted volumes
   * Total/free/used space
   * Filesystem information
   * Mount path
   * Read-only status
   * Make the existing `Volumes` component useful rather than decorative

4. **Scan robustness**

   * Permission-denied directories
   * Symlinks
   * Broken symlinks
   * Files disappearing during scan
   * Very large directories
   * Cancellation
   * Proper error/warning aggregation

5. **Scan cancellation**

   * Add a Cancel button
   * Propagate cancellation from Vue → Tauri → C++
   * Make sure the sidecar terminates cleanly
   * Return a distinct cancelled state rather than an error

6. **Performance pass**

   * Benchmark against large directory trees
   * Avoid unnecessary Vue reactivity
   * Limit expensive rendering to useful datasets
   * Profile C++ traversal
   * Check memory usage
   * Test scans with hundreds of thousands/millions of entries

7. **UX polish**

   * Empty states
   * Skeleton/loading states
   * Better error presentation
   * Responsive layout
   * Tooltips
   * Keyboard navigation
   * Consistent icons
   * Dark mode if we decide it belongs in v1

8. **Testing**

   * C++ unit/integration tests
   * Rust/Tauri command tests where practical
   * Vue component tests
   * Scanner composable tests
   * End-to-end smoke test:

   ```text
   Select directory
        ↓
   Scan
        ↓
   Progress
        ↓
   Dashboard
        ↓
   Drill down
        ↓
   Breadcrumb back
        ↓
   Cancel/error handling
   ```

9. **Packaging and release**

   * Linux `.deb` / AppImage
   * Windows installer
   * macOS bundle later
   * Verify sidecar binaries for each target
   * CI builds
   * Versioning
   * Release artifacts

### Then v2

Only after v1 is stable would I add the more ambitious features:

```text
Disk Analyzer v1
├── Directory scanning
├── Treemap
├── Drill-down
├── Breadcrumb navigation
├── Largest files
├── File-type analysis
├── Volume analysis
├── Cancellation
├── Error handling
└── Cross-platform packaging

Disk Analyzer v2
├── File search
├── Duplicate detection
├── Similar files
├── Cleanup recommendations
├── Delete/move operations
├── Exclusion rules
├── Historical scan comparison
├── Scan caching
└── Background monitoring
```

The key is **not to let v1 become bloated**. Once we finish file exploration, file types, volumes, cancellation, robustness, performance, tests, and packaging, we should be able to call **Disk Analyzer v1 production-ready** and then use the same C++/Vue/Tauri architecture for the next System Analyzer module.
