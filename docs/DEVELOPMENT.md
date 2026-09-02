# Development Log

## Cross-Platform Packaging (Disk Analyzer v1)

Status: Implemented (Linux/Windows bundling verified via CI)

### Target matrix

| OS | Architecture | Bundles | Notes |
| ---- | -------------- | --------- | ------- |
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

---

## Network Analyzer: Phase 0 Architecture & Contract

> See also: [`docs/Network-analyzer-plan.md`](Network-analyzer-plan.md) for the high-level
> phase sequence. This section is the **concrete architecture and wire contract** that
> Phases 0.1–0.4 lock down before any files are created.

Network Analyzer reuses the exact same three-layer stack — **C++20 engine / Tauri 2 shell /
Vue 3 frontend** — but introduces one fundamental divergence from Disk Analyzer:

> **Disk Analyzer: one-shot sidecar.** The binary runs once, writes a single JSON document
> to stdout, and exits. Tauri spawns → waits → collects stdout.
>
> **Network Analyzer: long-lived sidecar.** The binary stays running, reads JSON-Lines
> commands from stdin, and writes JSON-Lines responses (snapshots) to stdout. Tauri spawns
> once, then drives the sidecar through commands and consumes its stream of responses.

This is necessary because network monitoring is inherently **continuous**: we sample
interface counters at regular intervals, compute throughput deltas, and persist rollups —
none of which makes sense as a per-scan fork/exec cycle.

### Architecture diagram

```text
┌──────────────────────────────────────────────────────────────┐
│  Vue 3 frontend (apps/web)                                   │
│                                                              │
│  Network tab: live chart · throughput · interface list      │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             │ Tauri events  (network-interfaces,
                             │ network-snapshot, network-stop)
                             │ Tauri invoke  (network_start,
                             │                  network_stop,
                             │                  network_history)
                             │
┌────────────────────────────▼─────────────────────────────────┐
│  Tauri 2 desktop shell                                       │
│  apps/desktop/src-tauri/src/lib.rs                           │
│                                                              │
│  NetworkMonitor state:                                       │
│    • CommandChild (the running --network sidecar)          │
│    • RingBuffer of recent snapshots (in-process copy)      │
│                                                              │
│  Commands:                                                   │
│    • network_start  → spawn sidecar(--network)              │
│    • network_stop   → send {"command":"shutdown"} on stdin   │
│    • network_history → query SQLite via sidecar command      │
│                                                              │
│  Events forwarded to Vue:                                    │
│    • network-interfaces  (one-time at startup)               │
│    • network-snapshot    (every sampling interval)           │
│    • network-error       (diagnostics / failures)            │
└────────────────────────────┬─────────────────────────────────┘
                             │
                             │ stdout: JSON-Lines  ──────────────┐
                             │ stdin:  JSON-Lines command         │
                             │ stderr: diagnostics                │
                             │                                      │
┌────────────────────────────▼─────────────────────────────────┤
│  C++20 network engine                                        │
│  cpp/src/app/NetworkApp.cpp  ← long-lived command loop         │
│  cpp/src/core/NetworkSampler.cpp  ← sampling + throughput       │
│  cpp/src/core/RingBuffer.hpp  ← fixed-size sample store       │
│  cpp/src/serialization/               ← JSON-Lines serializer  │
│  cpp/src/platform/linux/LinuxNetworkProvider.cpp              │
│  cpp/src/platform/windows/WindowsNetworkProvider.cpp (Phase 4)│
│  cpp/src/platform/macos/MacOSNetworkProvider.cpp   (Phase 5)  │
│                                                              │
│  Database: SQLite (cpp/third_party/sqlite3/)                 │
│  Rollup target: network_rollups table                        │
│                                                              │
│  main.cpp branches:                                           │
│    system-analyzer <directory>  → Disk Analyzer (one-shot)   │
│    system-analyzer --network    → Network Analyzer (long-lived)│
└──────────────────────────────────────────────────────────────┘
```

---

### 0.1 — Sidecar Lifecycle: The Long-Lived Command Protocol

#### Mode selection in main.cpp

The single `system-analyzer` binary now supports two modes, selected by argv:

| Invocation | Mode | Lifetime | Command input | Response output |
|---|---|---|---|---|
| `system-analyzer <directory>` | Disk scan | one-shot | CLI arg | one JSON document on stdout |
| `system-analyzer --network` | Network monitor | long-lived | JSON-Lines on stdin | JSON-Lines on stdout |

```cpp
// cpp/src/main.cpp — addition
int main(int argc, char *argv[]) {
    // ...existing disk-scan path (argc == 2, non-flag arg)...

    if (argc == 2 && std::string(argv[1]) == "--network") {
        return NetworkApp::run();   // long-lived loop, never returns until shutdown
    }

    std::cerr << "Usage: system-analyzer <directory> | --network\n";
    return 1;
}
```

#### JSON-Lines command protocol

All communication across stdin and stdout is **JSON-Lines**: one complete JSON object
per line, terminated by `\n`. This is deliberately simple — no framing, no length prefix —
because each line is self-contained and can be parsed independently. `nlohmann::json`
is already vendored for exactly this purpose.

**Command grammar** (sent from Tauri on stdin):

```json
{"command": "network_snapshot"}
{"command": "set_interval", "interval_ms": 500}
{"command": "history", "interface_id": "eth0", "since_ts": 1700000000, "limit": 100}
{"command": "shutdown"}
```

| Command | Parameters | Sidecar response |
|---|---|---|
| `network_snapshot` | *(none)* | Immediate `network_snapshot` message on stdout |
| `set_interval` | `interval_ms` (int) | `interval_set` acknowledgement |
| `history` | `interface_id` (str, optional), `since_ts` (int, optional), `limit` (int) | `history_response` with rollup rows |
| `shutdown` | *(none)* | `shutdown_ack`, then clean exit (code 0) |

**Response grammar** (sent from sidecar on stdout), every message has a `type` field:

```json
{"type": "interfaces", "ts": 1700000000, "interfaces": [...]}
{"type": "network_snapshot", "ts": 1700000001, "snapshot": {...}}
{"type": "interval_set", "interval_ms": 500}
{"type": "history_response", "rows": [...], "total": 42}
{"type": "shutdown_ack"}
{"type": "error", "message": "human-readable error"}
```

#### Command loop (conceptual)

```cpp
// cpp/src/app/NetworkApp.cpp
int NetworkApp::run() {
    auto provider = platform::createNetworkProvider();

    // 1. Discover interfaces immediately → emit "interfaces" message
    auto interfaces = provider->discoverInterfaces();
    stdout() << serialize(InterfacesMessage{interfaces});

    // 2. Open SQLite at the user data dir
    Database db = Database::open(userDataDir() / "network-analyzer.db");
    db.exec("CREATE TABLE IF NOT EXISTS network_rollups ...");

    // 3. Enter command loop
    std::string line;
    while (std::getline(std::cin, line)) {
        auto cmd = nlohmann::json::parse(line);
        if (cmd["command"] == "network_snapshot") {
            auto snapshot = provider->collectSnapshot(interfaces);
            db.insertRollups(snapshot);        // Phase 1 (0.2)
            stdout() << serialize(SnapshotMessage{snapshot});
        } else if (cmd["command"] == "shutdown") {
            stdout() << serialize(ShutdownAckMessage{});
            break;
        }
    }
    return 0;
}
```

> **Design note — proactive vs. reactive sampling.** The initial implementation (Phase 0.4)
> uses a **timer-driven** sampler: the sidecar samples on its own at a fixed interval (1
> second default) and emits snapshots autonomously. Commands (`network_snapshot`) are
> available as an on-demand override. This means the sidecar produces data even when the
> Vue Network tab is closed, satisfying Phase 0.3's "continue sampling" requirement.

#### Sidecar stdin write from Tauri

`tauri_plugin_shell::process::CommandChild` exposes a `write()` method in the version
pinned in `Cargo.toml` (`tauri-plugin-shell = "2.3.5"`). The Tauri side sends commands by
writing JSON-Lines to `child.stdin`. If stdin writes fail (pipe broken), Tauri kills the
process and emits a `network-error` event.

#### Clean shutdown

Shutdown is **cooperative**, not a `kill()`:

1. Tauri writes `{"command":"shutdown"}\n` to the sidecar's stdin.
2. Sidecar writes `{"type":"shutdown_ack"}\n` to stdout, flushes, closes SQLite, exits 0.
3. Tauri waits for `Terminated` event, then drops the handle.

If the sidecar does not exit within 3 seconds of the shutdown command, Tauri calls
`child.kill()` as a fallback.

#### Coexistence with Disk Analyzer

The `--network` flag routes to a completely separate code path in `main.cpp`. The existing
one-shot disk scan code path is **unchanged**. The C++ engine binary simply grows a new
`else if` branch. CMake does not need separate targets — it is the same
`system-analyzer` executable with a new `main.cpp` branch and additional source/object
files.

---

### 0.2 — Persistence Schema: `network_rollups`

#### Dependency: SQLite

The C++ engine currently has **zero** runtime dependencies except `nlohmann/json`
(vendored in `cpp/third_party/`). Network Analyzer requires **SQLite3** for rollup
persistence. Two options:

| Option | Pros | Cons |
|---|---|---|
| System `libsqlite3` (CMake `find_package`) | No vendoring, standard | Not available on all CI images; needs `apt install libsqlite3-dev` |
| Vendored amalgamation (`cpp/third_party/sqlite3/sqlite3.c`) | Zero external deps, matches nlohmann pattern | Larger binary, needs upstream license file |

**Decision for Phase 1:** Use the **vendored amalgamation**. Drop `sqlite3.c` / `sqlite3.h`
into `cpp/third_party/sqlite3/` (same pattern as `nlohmann/json.hpp`), set
`SQLITE_OMIT_LOAD_EXTENSION=1` for security, and add it to `CMakeLists.txt` as a static
library target `sqlite3_static`. This keeps the build self-contained — consistent with how
`nlohmann/json` is already vendored.

The database file lives at the Tauri user-data directory (on Linux:
`$XDG_DATA_HOME/system-analyzer/` or `$HOME/.local/share/system-analyzer/`), which the
sidecar receives via an env var or CLI arg:

```text
system-analyzer --network --db-path <path>
```

#### Schema (stable for Phase 1–5)

```sql
CREATE TABLE IF NOT EXISTS network_rollups (
    ts              INTEGER NOT NULL,          -- Unix epoch seconds
    interface_id    TEXT    NOT NULL,          -- stable interface identifier (name)
    bytes_received  INTEGER NOT NULL,          -- cumulative rx bytes at sample time
    bytes_sent      INTEGER NOT NULL,          -- cumulative tx bytes at sample time
    PRIMARY KEY     (ts, interface_id)
);

-- Index for efficient range queries by time.
CREATE INDEX IF NOT EXISTS idx_network_rollups_ts
    ON network_rollups (ts);
```

**Why cumulative bytes, not deltas?** Cumulative counters are what the kernel exposes
(`/proc/net/dev`, `GetIfTable2`, `getifaddrs`). Storing them directly means we can
always recompute throughput for any historical interval: `(bytes_sent[t2] -
bytes_sent[t1]) / (t2 - t1)`. Deltas would lose this ability.

**What is deliberately NOT in the schema (Phase 0.2):**

```sql
-- DEFERRED to Phase 6 (application attribution):
-- app_usage_rollups   (per-process network accounting — needs different OS APIs)
```

The plan is explicit: introduce `app_usage_rollups` only after Phase 6 establishes the
application identity model. This keeps the initial schema stable and unchanging.

> **Phase 6 status (6.5 green):** the Linux attribution mechanism has been investigated,
> decided, and **implemented and tested**. `docs/phase6-linux-attribution.md` documents the
> mechanism survey, kernel evidence, and identity model. The implementation is live:
> `IApplicationNetworkProvider` (platform-neutral), `LinuxApplicationNetworkProvider`
> (`/proc/net/tcp` → inode → PID → canonical exe, with per-socket cumulative byte counters
> read via the SOCK_DIAG/`INET_DIAG_INFO` netlink dump — the same mechanism `ss -eit`
> uses; `TCPDIAG_GETSOCK` is rejected with EINVAL so `SOCK_DIAG_BY_FAMILY` + a
> until-`NLMSG_DONE` receive loop are used). It reports cumulative counters and calls
> `createApplicationNetworkProvider()` (Linux returns the implementation; Windows/macOS
> return nullptr, Linux-first). The `app_usage_rollups` migration (Phase 6.6) is **still
> deferred** until Step 6.5 gates are proven — same-uid TCP only, with graceful skip for
> permission/visibility failures; UDP and foreign-uid are surfaced as gaps, not
> approximations.

#### Write semantics

Each `network_snapshot` is processed as a single transaction:

1. Read all interface counters.
2. For each interface, `INSERT OR REPLACE INTO network_rollups VALUES (...)`.
3. Commit.

If the database is read-only or unavailable, the snapshot is still emitted on stdout
(the live chart works without persistence), and an error is written to stderr. The sidecar
does **not** crash on DB failure.

---

### 0.3 — Monitoring Semantics

#### What "history" means

```text
Application running
        │
        ▼
Network monitoring active
        │
        ├── Network tab open
        │       └── live visualization (ring buffer → Vue chart)
        │
        └── Network tab closed
                └── continue sampling (ring buffer full, oldest evicted;
                    rollups still written to SQLite)
```

**Closing the application stops monitoring.** The sidecar is a child process of the Tauri
desktop shell; when the app exits, the sidecar is terminated (or shut down cooperatively
via the `shutdown` command in §0.1).

> **"24-hour history" = the last 24 hours of observed application runtime**, not
> guaranteed continuous system-wide history. If the user closes the app at 3 PM and
> reopens at 9 AM the next day, the history will show a gap from 3 PM to 9 AM. The UI
> **must not interpolate** across that gap — it should render it visibly.

#### Gap representation in the UI

The Vue chart must explicitly represent gaps. When the most recent sample is older than
`2 × sampling_interval`, the chart shows a "gap" indicator (dashed line, faded region, or
explicit "no data" band) rather than connecting stale points. This is a **UI contract
requirement**, not just a visual nicety.

#### Sampling interval

| Context | Interval |
|---|---|
| Default running | 1000 ms (1 second) |
| When app is minimized or tab is backgrounded | configurable, 2000–5000 ms |
| When Tab is in focus | 500 ms (smoother live chart) |

The interval is controlled via the `set_interval` command. Phase 0.4 uses a fixed 1-second
interval; dynamic adjustment is a Phase 3 enhancement.

---

### 0.4 — First Vertical Slice

After Phases 0.1–0.3 are locked, we build the first end-to-end slice:

```text
Linux (/proc/net/dev)
  ↓
INetworkProvider → NetworkSampler
  ↓
Long-lived C++ sidecar (--network)
  ↓
SQLite network_rollups
  ↓
Tauri NetworkMonitor state
  ↓
RingBuffer + events → Vue
  ↓
Live chart + network summary
```

#### In scope

- **Interface discovery** on Linux via `getifaddrs()` + `/sys/class/net/<iface>/`.
  Fields: `id`, `name`, `mac`, `isUp`, `isLoopback`, `mtu`, `speed`.
- **RX/TX counters** from `/proc/net/dev` (parsed line by line).
- **Throughput** computed as the delta between consecutive samples divided by elapsed time.
- **Live chart** in Vue (Canvas-based, using `requestAnimationFrame` or Chart.js — TBD).
- **Basic network summary**: total RX/s, total TX/s, active interface count, top talker.
- **SQLite rollup** writes on every sample.
- **Tauri state management**: spawn once, forward events, graceful shutdown.

#### Out of scope (Phase 0.4)

- Windows / macOS providers (Phases 4–5).
- Application attribution / `app_usage_rollups` (Phase 6).
- Historical query UI / trend analysis (Phase 8).
- Background tray monitoring (Phase 10).
- Interval changes, history queries via the command protocol (start with fixed interval,
  no history command UI).

#### First vertical slice gate

The slice passes when all of these are true:

1. `system-analyzer --network` starts, discovers interfaces, and emits a valid
   `interfaces` message on stdout.
2. After ~1 second, it emits a `network_snapshot` message with correct cumulative
   counters for every non-loopback interface.
3. The `network_rollups` table receives a row per interface per sample (verify by
   querying the SQLite file directly).
4. Tauri spawns the sidecar, forwards `interfaces` and `network_snapshot` events to
   Vue, and renders a live chart.
5. On app shutdown, Tauri sends `shutdown`, the sidecar responds with `shutdown_ack`,
   closes SQLite cleanly, and exits 0.
6. The C++ unit test `network-schema-contract` (new ctest, analogous to
   `scan-schema-contract`) validates the JSON message schemas on Linux.

---

### C++ Domain Model (new files)

Following the Disk Analyzer pattern (`domain/*.hpp` = pure structs, no platform deps):

```text
cpp/include/system_analyzer/domain/
├── NetworkInterface.hpp      ← static interface description
├── NetworkStats.hpp          ← raw kernel counters (one per interface)
├── InterfaceSample.hpp       ← stats + derived throughput (per sample)
├── NetworkSnapshot.hpp       ← point-in-time collection of all InterfaceSamples
├── NetworkSummary.hpp        ← aggregate totals + top interface
└── NetworkRollup.hpp         ← one row for network_rollups table
```

**NetworkInterface** (`domain/NetworkInterface.hpp`):

```cpp
struct NetworkInterface {
    std::string id;          // stable identifier (interface name, e.g. "eth0")
    std::string name;        // display name (same as id on Linux/macOS)
    std::string mac;         // empty if no MAC (e.g. loopback)
    std::uint32_t mtu = 0;
    std::int64_t  speed = -1; // bits/sec, -1 if unknown
    bool isUp = false;
    bool isLoopback = false;
    std::vector<std::string> ipv4;  // populated Phase 6
    std::vector<std::string> ipv6;  // populated Phase 6
};
```

**NetworkStats** (`domain/NetworkStats.hpp`):

```cpp
struct NetworkStats {
    std::uint64_t rxBytes   = 0;
    std::uint64_t txBytes   = 0;
    std::uint64_t rxPackets = 0;
    std::uint64_t txPackets = 0;
    std::uint64_t rxErrors  = 0;
    std::uint64_t txErrors  = 0;
    std::uint64_t rxDropped = 0;
    std::uint64_t txDropped = 0;
};
```

**InterfaceSample** (`domain/InterfaceSample.hpp`):

```cpp
struct InterfaceSample {
    std::string interfaceId;
    NetworkStats stats;
    double rxBytesPerSec   = 0.0;  // throughput since previous sample
    double txBytesPerSec   = 0.0;
    double rxPacketsPerSec = 0.0;
    double txPacketsPerSec = 0.0;
};
```

**NetworkSnapshot** (`domain/NetworkSnapshot.hpp`):

```cpp
struct NetworkSnapshot {
    std::uint64_t ts;                     // Unix epoch seconds
    std::vector<InterfaceSample> interfaces;
};
```

**NetworkSummary** (`domain/NetworkSummary.hpp`):

```cpp
struct NetworkSummary {
    std::uint64_t ts;
    double totalRxBytesPerSec = 0.0;
    double totalTxBytesPerSec = 0.0;
    std::uint32_t activeInterfaceCount = 0;
        std::string topTalker;               // interface_id of highest total throughput
};
```

### C++ Core Layer (new files)

Mirroring the Disk Analyzer's `core/` pattern (abstract interfaces + platform-independent
utilities):

```text
cpp/include/system_analyzer/core/
├── INetworkProvider.hpp       ← abstract interface (like IFileScanner / IVolumeProvider)
├── NetworkSampler.hpp         ← throughput calculation + sample orchestration
├── NetworkRingBuffer.hpp      ← fixed-size ring buffer of InterfaceSample snapshots
└── NetworkContext.hpp         ← sampling callbacks (like ScanContext)
```

**INetworkProvider** (`core/INetworkProvider.hpp`):

```cpp
class INetworkProvider {
public:
    virtual ~INetworkProvider() = default;

    // One-time discovery at startup.
    [[nodiscard]] virtual std::vector<domain::NetworkInterface>
    discoverInterfaces() const = 0;

    // Collect current raw counters for a known set of interfaces.
    [[nodiscard]] virtual std::vector<domain::NetworkStats>
    collectStats(
        const std::vector<domain::NetworkInterface>& interfaces
    ) const = 0;
};
```

**NetworkSampler** (`core/NetworkSampler.hpp`):

Owns the previous-sample state, computes throughput deltas, and assembles a
`NetworkSnapshot` from `INetworkProvider` output:

```cpp
class NetworkSampler {
public:
    domain::NetworkSnapshot sample(
        const std::vector<domain::NetworkInterface>& interfaces,
        const INetworkProvider& provider
    );
    // Internally: compares current stats against previousSample_,
    // computes per-interface deltas / elapsed seconds → *PerSec fields.
private:
    std::optional<std::vector<NetworkStats>> previousStats_;
    std::chrono::steady_clock::time_point previousTime_;
};
```

**NetworkRingBuffer** (`core/NetworkRingBuffer.hpp`):

A fixed-capacity circular buffer of the most recent `NetworkSnapshot` objects. Serves
as the in-memory data source for the Vue live chart. Old snapshots are evicted when
full. This is a simple template class — same pattern as `DirectorySizeAggregator`
being a small, self-contained, testable utility.

**NetworkContext** (`core/NetworkContext.hpp`):

```cpp
struct NetworkContext {
    std::function<void(std::uint64_t)> onSample;  // called after each snapshot
    std::function<bool()> isCancelled;            // checked between samples
};
```

#### Platform layer (new files)

```text
cpp/include/system_analyzer/platform/
├── network/factory.hpp        ← createNetworkProvider()
├── linux/LinuxNetworkProvider.hpp
├── windows/WindowsNetworkProvider.hpp   (Phase 4)
└── macos/MacOSNetworkProvider.hpp       (Phase 5)
```

**LinuxNetworkProvider** reads from:
- `getifaddrs()` for interface names + flags + MAC.
- `/sys/class/net/<iface>/speed`, `/sys/class/net/<iface>/mtu`, `/sys/class/net/<iface>/operstate`.
- `/proc/net/dev` for cumulative RX/TX counters.

This mirrors how `LinuxVolumeProvider` reads `/proc/self/mounts` + `statvfs` and
`LinuxDiskUsageProvider` reads `statvfs` — **one OS native API per concern, all behind
the factory**.

#### Application layer

```text
cpp/include/system_analyzer/app/
├── Application.hpp          ← existing (disk scan, unchanged)
└── NetworkApp.hpp           ← new: long-lived command loop
```

and an SQLite wrapper for rollup persistence.

#### Serialization

```text
cpp/include/system_analyzer/serialization/
├── ScanResultSerializer.hpp     ← existing (unchanged)
└── NetworkSnapshotSerializer.hpp ← new
```

Uses `nlohmann::json` (already vendored) to serialize `InterfacesMessage`,
`SnapshotMessage`, `ShutdownAckMessage`, `ErrorMessage`, etc. All field names follow
the **camelCase** convention established by `ScanResultSerializer`.

#### CMake Integration

##### New SQLite target

```cmake
# Add to CMakeLists.txt, near the top
add_library(sqlite3_static STATIC cpp/third_party/sqlite3/sqlite3.c)
target_include_directories(sqlite3_static PUBLIC cpp/third_party/sqlite3)
target_compile_definitions(sqlite3_static PRIVATE SQLITE_OMIT_LOAD_EXTENSION)
```

##### New engine sources

The `system-analyzer` executable gains NetworkApp sources but **only on non-Windows** for
now (Linux-only Phase 0.4). Windows/macOS network providers are added in Phases 4–5.

```cmake
# Extend ENGINE_SOURCES conditionally
if(NOT WIN32)
    list(APPEND ENGINE_SOURCES
        cpp/src/app/NetworkApp.cpp
        cpp/src/core/NetworkSampler.cpp
        cpp/src/serialization/NetworkSnapshotSerializer.cpp
        cpp/src/platform/${SYSTEM_ANALYZER_PLATFORM_DIR}/...NetworkProvider.cpp
    )
endif()
```

> **Important:** The `system-analyzer` binary is shared. When compiled on Linux, it
> supports both `<directory>` (disk) and `--network`. When compiled on Windows (during
> Phase 4), it still only supports disk scan (network not yet compiled in). The `--network`
> branch in `main.cpp` is guarded to avoid linking unused symbols on platforms that
> haven't implemented the provider yet.

##### New contract test

```cmake
add_executable(network-schema-contract-test
    cpp/tests/contract/NetworkSchemaContractTest.cpp
    ${ENGINE_SOURCES}   # same engine sources
)
target_include_directories(network-schema-contract-test PRIVATE cpp/include cpp/third_party)
add_test(NAME network-schema-contract COMMAND network-schema-contract-test)
```

The test mirrors `ScanSchemaContractTest.cpp`: it constructs a `NetworkSnapshot` in
memory, serializes it, parses the JSON, and asserts field names, types, and numeric
domains. Unlike the disk test, it does **not** need filesystem fixtures — network
counters can be synthetic.

---

### JSON Wire Protocol (Concrete Schema)

Every message on stdout and stdin is a single line of JSON terminated by `\n`. Below is
the **stable contract** — field names and types must not change without a version bump.

#### Message: `interfaces` (sidecar → Tauri, emitted once at startup)

```json
{
  "type": "interfaces",
  "ts": 1700000000,
  "interfaces": [
    {
      "id": "eth0",
      "name": "eth0",
      "mac": "00:11:22:33:44:55",
      "mtu": 1500,
      "speed": 1000000000,
      "isUp": true,
      "isLoopback": false,
      "ipv4": [],
      "ipv6": []
    }
  ]
}
```

#### Message: `network_snapshot` (sidecar → Tauri, emitted every interval)

```json
{
  "type": "network_snapshot",
  "ts": 1700000001,
  "snapshot": {
    "totalRxBytesPerSec": 1234.56,
    "totalTxBytesPerSec": 789.12,
    "activeInterfaceCount": 2,
    "topTalker": "eth0",
    "interfaces": [
      {
        "interfaceId": "eth0",
        "stats": {
          "rxBytes": 1234567890,
          "txBytes": 9876543210,
          "rxPackets": 12345,
          "txPackets": 67890,
          "rxErrors": 0,
          "txErrors": 0,
          "rxDropped": 0,
          "txDropped": 0
        },
        "rxBytesPerSec": 1234.56,
        "txBytesPerSec": 789.12,
        "rxPacketsPerSec": 12.5,
        "txPacketsPerSec": 6.3
      }
    ]
  }
}
```

#### Message: `shutdown_ack` (sidecar → Tauri)

```json
{
  "type": "shutdown_ack"
}
```

#### Message: `error` (sidecar → Tauri, on failure)

```json
{
  "type": "error",
  "message": "Failed to open /proc/net/dev: Permission denied"
}
```

#### Message: `history_response` (sidecar → Tauri, reply to `history` command)

```json
{
  "type": "history_response",
  "total": 42,
  "rows": [
    {"ts": 1700000000, "interface_id": "eth0", "bytes_received": 1000, "bytes_sent": 500}
  ]
}
```

#### Message: `interval_set` (sidecar → Tauri, reply to `set_interval`)

```json
{
  "type": "interval_set",
  "interval_ms": 500
}
```

---

### Tauri Integration

#### State

```rust
// apps/desktop/src-tauri/src/lib.rs — addition
struct NetworkMonitor {
    /// The running sidecar process (None when stopped).
    child: Mutex<Option<CommandChild>>,
}
```

This replaces the disk `CurrentScan` pattern with a persistent handle. There is no
`cancel_requested` flag — instead, Tauri sends `shutdown` on stdin and waits for the
`Terminated` event.

#### Commands

| Tauri command | Action |
|---|---|
| `network_start` | Spawn `system-analyzer --network --db-path <path>`, register stdout listener |
| `network_stop` | Write `{"command":"shutdown"}` to stdin, await termination |
| `network_history` | Write `{"command":"history",...}` to stdin, await `history_response` on stdout |

#### stdout → event forwarding

The stdout listener reads line-delimited JSON. Each line is parsed and routed by
`type`:

```rust
while let Some(event) = receiver.recv().await {
    match event {
        CommandEvent::Stdout(bytes) => {
            for line in split_lines(&bytes) {
                let msg: serde_json::Value = serde_json::from_str(&line)?;
                match msg["type"].as_str() {
                    Some("interfaces")      => app.emit("network-interfaces", msg),
                    Some("network_snapshot") => app.emit("network-snapshot", msg),
                    Some("shutdown_ack")    => /* terminate */,
                    Some("error")           => app.emit("network-error", msg),
                    _ => {}
                }
            }
        }
        CommandEvent::Error(e) => app.emit("network-error", e),
        CommandEvent::Terminated(_) => {
            if !was_shutdown { app.emit("network-error", "sidecar died unexpectedly"); }
        }
        _ => {}
    }
}
```

> **Line-buffering concern.** The C++ sidecar must `flush()` stdout after every message.
> Tauri's stdout event delivers chunks of bytes that may straddle message boundaries, so
> the Rust side must buffer incomplete lines until `\n` is seen. This is the standard
> pattern for JSON-Lines over pipes.

#### db-path

The database path is resolved by Tauri via `app.path().app_data_dir()` and passed to
the sidecar as a CLI arg: `--db-path <path>`. This keeps the C++ engine free of
platform-specific path logic.

---

### Vue Integration

#### Types (`apps/web/src/types/network.ts`)

Exact mirror of the JSON wire protocol, following the same `scan.ts` pattern:

```ts
export interface NetworkInterface {
  id: string;
  name: string;
  mac: string;
  mtu: number;
  speed: number;       // bits/sec, -1 if unknown
  isUp: boolean;
  isLoopback: boolean;
  ipv4: string[];
  ipv6: string[];
}

export interface NetworkStats {
  rxBytes: number;
  txBytes: number;
  rxPackets: number;
  txPackets: number;
  rxErrors: number;
  txErrors: number;
  rxDropped: number;
  txDropped: number;
}

export interface InterfaceSample {
  interfaceId: string;
  stats: NetworkStats;
  rxBytesPerSec: number;
  txBytesPerSec: number;
  rxPacketsPerSec: number;
  txPacketsPerSec: number;
}

export interface NetworkSnapshot {
  ts: number;
  interfaces: InterfaceSample[];
}

export interface NetworkSummary {
  ts: number;
  totalRxBytesPerSec: number;
  totalTxBytesPerSec: number;
  activeInterfaceCount: number;
  topTalker: string;
}
```

#### Service (`apps/web/src/services/network.ts`)

Thin wrappers around `invoke()`, mirroring `services/analyzer.ts`:

```ts
export async function startNetworkMonitor(dbPath: string): Promise<void> {
  await invoke('network_start', { dbPath });
}

export async function stopNetworkMonitor(): Promise<void> {
  await invoke('network_stop');
}

export async function fetchNetworkHistory(
  opts?: { interfaceId?: string; sinceTs?: number; limit?: number }
): Promise<NetworkRollup[]> {
  return await invoke('network_history', opts);
}
```

#### Composable (`apps/web/src/composables/useNetworkMonitor.ts`)

Mirrors `useScanner.ts` but manages persistent state (no "scan" lifecycle, just start/stop):

```ts
export function useNetworkMonitor() {
  const interfaces = ref<NetworkInterface[]>([]);
  const snapshots = ref<NetworkSnapshot[]>([]);    // ring-buffered in Vue (keep last N)
  const summary = ref<NetworkSummary | null>(null);
  const isMonitoring = ref(false);
  const error = ref<string | null>(null);

  // Listen for Tauri events
  // network-interfaces → set interfaces.value
  // network-snapshot  → push to snapshots, compute summary
  // network-error     → set error

  return { interfaces, snapshots, summary, isMonitoring, error, start, stop };
}
```

#### Live chart component

A `NetworkChart.vue` component renders the ring-buffered throughput data as a live
Canvas chart. It subscribes to the `useNetworkMonitor` composable and re-renders on each
new snapshot. Gap detection: if `snapshots[last].ts - snapshots[prev].ts > 2 × interval`,
render a gap marker rather than connecting the points.

---

### Contract Test Strategy

The existing `scan-schema-contract` test (ctest name `scan-schema-contract`)
establishes the **precedent**: every platform builds a test binary that runs a real scan
through `Application` and asserts the JSON schema is shape-stable. Network Analyzer
follows the same pattern:

**Test name:** `network-schema-contract`

**What it validates:**

1. `NetworkSnapshot` serializes to JSON with the exact field names and types defined
   in §0.4 (camelCase, unsigned integers for byte counters, doubles for throughput,
   `interfaceId` as string, `ts` as integer).
2. Every `InterfaceSample` has all 8 throughput/counter fields present and correctly typed.
3. The `interfaces` discovery message has the expected array structure.
4. Synthetic counter deltas produce non-negative throughput values (throughput is
   computed from monotonically increasing counters; a decrease would indicate a
   counter reset, which must be handled, not produce negative throughput).
5. The `shutdown_ack` message has no payload fields.

**Not validated by the contract test:**

- Real OS data (values differ per machine, per load). The shape must be stable; values
  may not. This is the exact same principle documented for `scan-schema-contract`.
- Performance characteristics.
- Cross-platform data equivalence (the Linux provider reads `/proc/net/dev`; the Windows
  provider reads `GetIfTable2`; the macOS provider reads `getifaddrs` + `sysctl`. Same
  JSON shape, different counters — which is correct and expected).

---

### Phase 0 Summary & Gate

| Phase | Topic | Decision | File(s) to create |
|---|---|---|---|
| 0.1 | Sidecar lifecycle | Long-lived `--network` mode, JSON-Lines stdin/stdout | `main.cpp` branch, `NetworkApp` |
| 0.2 | Persistence schema | SQLite `network_rollups` only, vendored sqlite3 | `cpp/third_party/sqlite3/`, `CMakeLists.txt` |
| 0.3 | Monitoring semantics | App-runtime history, explicit gaps, not continuous | Document convention |
| 0.4 | First vertical slice | Linux-only: /proc/net/dev → live chart → SQLite | All C++ core/network files, Tauri `NetworkMonitor`, Vue `useNetworkMonitor` |

**Phase 0 gate:** The architecture and wire contract above are locked. No source files
are created yet. Implementation begins only after these decisions are recorded and
reviewed.

---

### Mapping: Disk Analyzer → Network Analyzer

| Disk Analyzer concept | Network Analyzer equivalent |
|---|---|
| `system-analyzer <dir>` (one-shot) | `system-analyzer --network` (long-lived) |
| `main.cpp` → one scan, one JSON output | `NetworkApp::run()` → command loop |
| `stdout` → single JSON `ScanResult` | `stdout` → JSON-Lines stream of messages |
| `stderr` → `PROGRESS:<n>` | `stderr` → free-form diagnostics |
| `IFileScanner` / `IVolumeProvider` / `IDiskUsageProvider` | `INetworkProvider` |
| `platform::createFileScanner()` etc. | `platform::createNetworkProvider()` |
| `scan-schema-contract` test | `network-schema-contract` test |
| `ScanResultSerializer` (nlohmann/json) | `NetworkSnapshotSerializer` (nlohmann/json) |
| `services/analyzer.ts` | `services/network.ts` |
| `composables/useScanner.ts` | `composables/useNetworkMonitor.ts` |
| `types/scan.ts` | `types/network.ts` |
| No persistence | SQLite `network_rollups` |
| Kill process = cancellation | `shutdown` command = clean exit |
| `CurrentScan` Tauri state | `NetworkMonitor` Tauri state |
| `PROGRESS:<n>` stderr → `scan-progress` event | JSON-Lines stdout → `network-snapshot` event |
| `/proc/self/mounts` + `statvfs` | `/proc/net/dev` + `getifaddrs()` |

The mapping is deliberately 1:1 — every Disk Analyzer component has a Network Analyzer
counterpart with the same layering, the same testing approach, and the same JSON-over-
pipe contract. The only structural change is lifespan: one-shot → long-lived.


---

## Phase 6.7 — Network Analyzer Read API + UI integration (COMPLETE)

Status: Implemented. Turns the Phase 6 collection/persistence pipeline into a
usable, queryable read API and a first live Network Analyzer screen.

### Rust — read-side API (6.7.1–6.7.4)

- **`network/monitor.rs`** (new) — `LiveTelemetry` + cloneable `MonitorHandle` +
  `run_monitor`: one long-lived sidecar, one shared `NetworkSampler`, a
  1-second tick that publishes an immutable live snapshot (ring-buffer copies);
  graceful 5s backoff respawn on crash. Runs for the app's lifetime,
  independent of any open page. `Database::persist_rollups` is reached through
  the shared `storage::DatabaseState` writer.
- **`storage/history.rs`** (new) — epoch-aligned read SQL over
  `network_rollups` / `app_usage_rollups`: `bucket_seconds_for_range`
  (≤1h→60s, ≤6h→300s, else→900s), per-total and per-interface aggregated
  series, per-app usage totals, top-N. `since` inclusive, `until` exclusive.
- **`network/readapi.rs`** (new) — camelCase DTOs + pure assembly from live
  snapshots (grouping, merged totals series) and history rows.
- **`lib.rs`** — the monitor is spawned once in the setup hook and the app
  manages a `MonitorHandle` + `storage::DatabaseState`; four commands exposed:
  `get_network_live`, `get_network_history`, `get_application_history`,
  `get_top_applications`. `DatabaseState` moved to `storage` so the monitor and
  the commands share the single writer; `tokio::time` added for the tick.

### Vue — Network Analyzer screen (6.7.5–6.7.6)

- `types/network.ts`, `services/network.ts`, `composables/useNetwork.ts`
  (live polling starts on mount and stops via `onUnmounted` while the Rust
  sampler keeps running — Phase 0 lifecycle), `formatRate` util.
- Components: `NetworkOverview` (RX/TX rates + explicit note that total
  interface traffic and attributable application traffic differ on Linux),
  `ThroughputChart` (dependency-free SVG live chart), `InterfacesPanel`,
  `TopApplications`, and `NetworkAnalyzer` orchestrator with a 1h/6h/24h range
  selector, live/history sections, loading + error + empty states for both.
- Wired into the layout nav (`Network`) and `App.vue` as a page-independent
  section of the dashboard.

### Verification (6.7.7)

- Rust: 51 tests (unit) + 1 sidecar integration — empty history, range bucket
  aggregation, `since`/`until` boundaries, per-interface series, top-app
  ranking + limit, live DTO grouping, monitor/sampler isolation. All passing.
- Frontend: `type-check` ✓, `oxlint` + `eslint` ✓, prettier-formatted ✓,
  `vite build` ✓, 41 vitest cases ✓ (service window mapping, composable poll
  lifecycle — including stop() leaves the sampler running, component
  telemetry/empty/loading/error states, `formatRate`).
- Cross-project regression: C++ ctest 14/14 ✓.

### Roadmap

```text
6.5 Linux application attribution       ✅
6.6 Application persistence + telemetry ✅
6.7 Read API + Network Analyzer UI      ✅ (this phase)
6.8 Hardening + cross-platform support  ← NEXT
6.9 Release/packaging validation
```

---
