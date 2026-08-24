# Network Analyzer Implementation Plan

Formalize Network Analyzer as a **phased implementation plan with explicit gates**, just like we did for Disk Analyzer. We should not jump between C++, Rust, and Vue randomly. Each phase should leave the repository in a buildable, testable state.

## Phase 0: Architecture and Contract

**Goal:** Establish the domain model and boundaries before touching OS-specific networking code.

### 0.1 Define network domain models

Create:

```text
cpp/include/system_analyzer/network/
├── NetworkInterface.hpp
├── NetworkSnapshot.hpp
├── ApplicationNetworkUsage.hpp
└── NetworkUsageProvider.hpp
```

Initial model:

```cpp
struct NetworkInterface {
    std::string id;
    std::string name;
    std::string displayName;
    bool isUp;
    bool isLoopback;
    uint64_t bytesReceived;
    uint64_t bytesSent;
};

struct ApplicationNetworkUsage {
    std::string id;
    std::string processName;
    std::string executablePath;
    uint32_t processId;
    uint64_t bytesReceived;
    uint64_t bytesSent;
};

struct NetworkSnapshot {
    uint64_t timestamp;
    std::vector<NetworkInterface> interfaces;
    uint64_t totalBytesReceived;
    uint64_t totalBytesSent;
    std::vector<ApplicationNetworkUsage> applications;
};
```

Keep the model platform-neutral.

### 0.2 Define provider interfaces

```text
NetworkUsageProvider
ApplicationNetworkUsageProvider
```

The key principle:

```text
OS implementation
      ↓
normalized domain model
      ↓
stable JSON
```

### 0.3 Define the JSON contract

Establish the exact serialized structure before implementing providers.

Add:

```text
network-schema-contract
```

The contract should validate:

* fields
* types
* required/optional properties
* enum values
* numeric ranges
* timestamp semantics
* interface identity
* application identity

### 0.4 Define sampling semantics

Lock these down:

```text
Sampling interval:       1 second
Live buffer:             5-10 minutes
Persistent bucket:       1 minute
Initial retention:       30 days
```

Define exactly what `bytesReceived` means:

**cumulative counter from the OS**, not "bytes during this sample."

Throughput is then calculated from counter deltas.

### Phase 0 gate

We do not proceed until:

```text
✓ Domain models compile
✓ Interfaces compile
✓ JSON serialization works
✓ Contract test passes
✓ No OS-specific code in domain layer
```

---

# Phase 1: Persistence Infrastructure

**Goal:** Establish durable telemetry storage before building the UI.

This belongs in the Tauri/Rust layer.

### 1.1 SQLite integration

Evaluate and integrate the SQLite implementation we choose for Tauri 2.

Create:

```text
apps/desktop/src-tauri/src/storage/
├── database.rs
├── migrations.rs
├── network_repository.rs
└── retention.rs
```

The storage layer should be generic enough to eventually support:

```text
Network history
Disk scan history
Future telemetry
```

### 1.2 Database schema

Start with:

```sql
CREATE TABLE network_rollups (
    ts INTEGER NOT NULL,
    interface_id TEXT NOT NULL,
    bytes_received INTEGER NOT NULL,
    bytes_sent INTEGER NOT NULL,
    PRIMARY KEY (ts, interface_id)
);

CREATE TABLE app_usage_rollups (
    ts INTEGER NOT NULL,
    app_id TEXT NOT NULL,
    process_name TEXT NOT NULL,
    executable_path TEXT,
    bytes_received INTEGER NOT NULL,
    bytes_sent INTEGER NOT NULL,
    PRIMARY KEY (ts, app_id)
);
```

Indexes:

```sql
CREATE INDEX idx_network_rollups_ts
ON network_rollups(ts);

CREATE INDEX idx_app_usage_rollups_ts
ON app_usage_rollups(ts);
```

### 1.3 SQLite configuration

Enable:

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
```

Configure the database for:

* single writer
* transactional rollups
* concurrent reads
* predictable failure handling

### 1.4 Repository API

Rust should expose something conceptually like:

```rust
insert_network_rollups(...)
insert_app_usage_rollups(...)
get_network_history(...)
get_app_usage_history(...)
delete_expired_rollups(...)
```

The frontend should never know that SQLite exists.

### 1.5 Retention

Implement:

```text
30 days
    ↓
automatic cleanup
```

Run retention cleanup:

* on application startup
* and/or periodically while monitoring

Make the retention period configurable internally so we can change it later without redesigning the schema.

### Phase 1 gate

```text
✓ SQLite initializes
✓ Migrations execute
✓ WAL enabled
✓ Insert/query works
✓ Transaction rollback works
✓ Retention works
✓ Rust tests pass
```

---

# Phase 2: Linux Network Provider

**Goal:** Implement the first real OS provider and establish the telemetry pipeline.

Linux is our development platform, so it should be first.

### 2.1 Interface discovery

Implement:

```text
LinuxNetworkUsageProvider
```

Read interface counters from the appropriate Linux system interfaces.

We need:

```text
interface name
operational state
RX bytes
TX bytes
loopback detection
```

Ignore irrelevant virtual interfaces initially where appropriate, but don't hard-code assumptions that would break Docker, VPNs, Wi-Fi, etc.

### 2.2 Counter normalization

Produce:

```json
{
  "interfaces": [
    {
      "id": "eth0",
      "name": "eth0",
      "displayName": "Ethernet",
      "isUp": true,
      "isLoopback": false,
      "bytesReceived": 123456,
      "bytesSent": 45678
    }
  ]
}
```

### 2.3 Throughput calculation

Do **not** calculate throughput inside the OS provider.

Provider:

```text
OS counters → cumulative bytes
```

Telemetry layer:

```text
counter(t)
counter(t-1)
      ↓
delta
      ↓
bytes/sec
```

This makes the provider easier to test.

### 2.4 Linux tests

Test:

* interface discovery
* RX/TX counters
* loopback filtering
* missing interfaces
* counter reset/wrap handling
* malformed system data

### Phase 2 gate

We should be able to execute something like:

```bash
system-analyzer network
```

and receive valid network JSON.

---

# Phase 3: Network Sampling Engine

**Goal:** Build the real-time telemetry pipeline.

At this point:

```text
C++
  ↓
NetworkSnapshot
  ↓
Tauri
  ↓
Sampler
```

### 3.1 One-second sampler

Tauri owns the sampling cadence:

```text
1s
 ↓
snapshot
 ↓
calculate delta
 ↓
live buffer
```

### 3.2 Ring buffer

Maintain approximately:

```text
600 samples
```

for a 10-minute window.

Each sample should contain:

```text
timestamp
download bytes/sec
upload bytes/sec
interface breakdown
application breakdown
```

### 3.3 Counter delta handling

Handle:

```text
normal increment
counter reset
interface disappearance
interface appearance
system sleep/resume
clock anomalies
```

Never produce a massive false throughput spike because the machine woke from sleep.

### 3.4 Rollup

Every 60 seconds:

```text
raw 1-second samples
        ↓
1-minute aggregate
        ↓
SQLite transaction
```

Write network and application rollups together.

### 3.5 Single writer

Make one telemetry persistence task responsible for SQLite writes.

```text
Sampler
   ↓
Telemetry channel
   ↓
Persistence worker
   ↓
SQLite
```

This keeps database ownership explicit.

### Phase 3 gate

```text
✓ 1-second sampling
✓ Live ring buffer
✓ Throughput calculation
✓ 60-second rollup
✓ SQLite persistence
✓ Restart retains history
```

---

# Phase 4: Windows Network Provider

**Goal:** Match Linux functionality with native Windows implementation.

Implement:

```text
WindowsNetworkUsageProvider
```

Use Windows networking APIs for interface enumeration and cumulative counters.

The output must conform to exactly the same domain model.

```text
Linux provider ───┐
                  ├──→ NetworkSnapshot
Windows provider ─┤
                  │
macOS provider ───┘
```

### Tests

Validate:

* interface discovery
* Ethernet
* Wi-Fi
* loopback
* disconnected interfaces
* cumulative counters
* provider failure behavior

### Phase 4 gate

The same network contract test must pass on Windows.

---

# Phase 5: macOS Network Provider

**Goal:** Complete the third platform implementation.

Implement:

```text
MacOSNetworkUsageProvider
```

Follow the same contract.

Test:

```text
Wi-Fi
Ethernet
Loopback
VPN
Interface changes
Sleep/wake
```

### Phase 5 gate

```text
Linux     ✓
Windows   ✓
macOS     ✓
```

All three produce compatible network snapshots.

---

# Phase 6: Application Network Usage

This is the **hardest phase**.

We should not pretend per-application accounting is equivalent across platforms.

### 6.1 Define application identity

Prefer:

```text
application ID
executable path
process name
PID
```

PID is useful for the current session but must not be used as the historical identity.

### 6.2 Linux

Investigate and select the appropriate mechanism for mapping network traffic to processes.

Potential mechanisms will need to account for:

```text
process
socket
connection
traffic
```

The implementation must handle permissions and processes that disappear while being inspected.

### 6.3 Windows

Use the appropriate Windows networking/process facilities to associate traffic with applications.

### 6.4 macOS

Implement the corresponding native mechanism and account for Apple's security and permission model.

### 6.5 Normalize

Every OS produces:

```cpp
ApplicationNetworkUsage
```

The UI does not care how the OS obtained it.

### 6.6 Aggregation

For each application:

```text
Chrome
  ↓
RX: 8.2 GB
TX: 1.1 GB
Total: 9.3 GB
```

### Phase 6 gate

At least:

```text
✓ Application discovery
✓ RX/TX attribution
✓ Stable application identity
✓ Permission handling
✓ Process disappearance handling
✓ Cross-platform contract
```

---

# Phase 7: Network Analyzer UI

Only after the telemetry backend is reliable.

Create:

```text
apps/web/src/
├── components/network/
│   ├── NetworkOverview.vue
│   ├── NetworkInterfaceCard.vue
│   ├── NetworkThroughputChart.vue
│   ├── NetworkHistory.vue
│   ├── ApplicationNetworkTable.vue
│   └── NetworkUsageSummary.vue
│
├── composables/
│   └── useNetworkMonitor.ts
│
└── types/
    └── network.ts
```

### Dashboard

```text
┌────────────────────────────────────────────┐
│ Network                                    │
├──────────────────┬─────────────────────────┤
│ Download         │ Upload                  │
│ 12.4 MB/s        │ 1.8 MB/s                │
├──────────────────┴─────────────────────────┤
│                                            │
│       Live throughput                      │
│                                            │
├────────────────────────────────────────────┤
│ Today's Usage                              │
│ Download  42.7 GB    Upload  8.3 GB        │
├────────────────────────────────────────────┤
│ Applications                               │
│                                            │
│ Chrome       32.4 GB                       │
│ Discord       4.2 GB                       │
│ VS Code       2.1 GB                       │
└────────────────────────────────────────────┘
```

### UI requirements

* Live throughput
* Current interface
* Upload/download totals
* 24-hour chart
* Application ranking
* Interface breakdown
* Empty state
* Permission warning
* Monitoring state
* Error state

---

# Phase 8: Historical Views

Now use the SQLite data properly.

### 8.1 24-hour view

Query:

```text
last 24 hours
↓
1-minute buckets
↓
chart
```

### 8.2 Application history

Allow:

```text
Chrome
  ↓
Today
  ↓
Downloaded: 32.4 GB
Uploaded:    4.8 GB
```

### 8.3 Interface history

```text
Wi-Fi
Ethernet
VPN
```

Each can be compared independently.

### Phase 8 gate

Close the application:

```text
monitor
 ↓
close
 ↓
reopen
 ↓
history still exists
```

That is the key durability test.

---

# Phase 9: Performance and Reliability

Before calling Network Analyzer production-ready:

### CPU

Measure:

```text
C++ provider CPU
Tauri sampler CPU
SQLite CPU
Vue rendering CPU
```

### Memory

Measure:

```text
ring buffer
SQLite connection
Vue chart history
application list
```

### Database

Test:

```text
1 day
7 days
30 days
```

Verify retention and database size.

### Stress scenarios

```text
No network
Many interfaces
VPN
Docker
Heavy download
Heavy upload
Many processes
Process exits during sample
Interface disappears
Laptop sleep
Network reconnect
Application restart
```

---

# Phase 10: Integration and Testing

Add the complete flow to the test matrix:

```text
Linux
├── C++ tests
├── provider tests
├── contract tests
├── Rust tests
└── Vue tests

Windows
├── C++ tests
├── provider tests
├── contract tests
├── Rust tests
└── Vue tests

macOS
├── C++ tests
├── provider tests
├── contract tests
├── Rust tests
└── Vue tests
```

Add an end-to-end smoke test:

```text
Start application
      ↓
Start network monitor
      ↓
Generate traffic
      ↓
Observe throughput
      ↓
Observe application attribution
      ↓
Wait for rollup
      ↓
Close application
      ↓
Reopen
      ↓
Verify historical data
```

---

# Phase 11: Documentation

Update:

```text
docs/DEVELOPMENT.md
docs/CPP-CHEATSHEET.md
docs/VUE-CHEATSHEET.md
README.md
```

Document:

* Network architecture
* Provider interfaces
* OS implementations
* SQLite schema
* Sampling strategy
* Rollup strategy
* Retention
* Permissions
* Known OS limitations
* Build instructions
* Troubleshooting

---

# Final Network Analyzer Architecture

When complete:

```text
                       SYSTEM ANALYZER
                              │
              ┌───────────────┴────────────────┐
              │                                │
        DISK ANALYZER                    NETWORK ANALYZER
              │                                │
              │                         NetworkSnapshot
              │                                │
              │                    ApplicationNetworkUsage
              │                                │
              │                                ▼
              │                             Tauri
              │                                │
              │                    ┌───────────┴───────────┐
              │                    │                       │
              │               Ring Buffer              SQLite
              │                    │                       │
              │                    ▼                       ▼
              │                Live UI              Historical UI
              │
              ▼
         C++ Engine
              │
      Platform Providers
              │
       ┌──────┼──────┐
       ▼      ▼      ▼
     Linux  Windows  macOS
```

## The execution rule

We follow the phases **in order**. At the end of every phase, we run its gate before moving forward. If implementation reveals that a decision needs changing, we modify the plan explicitly rather than silently drifting from it.

The first actual implementation task is therefore **Phase 0.1: define the Network domain models and provider interfaces**. We should inspect the current C++ architecture first so the new module follows the conventions already established by Disk Analyzer, then implement the interfaces and contract tests before writing any Linux networking code.
