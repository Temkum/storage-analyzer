# Network Analyzer Implementation Plan

Formalize Network Analyzer as a **phased implementation plan with explicit gates**, just like we did for Disk Analyzer. We should not jump between C++, Rust, and Vue randomly. Each phase should leave the repository in a buildable, testable state.s

## Revised Phase 0

### 0.1 Sidecar lifecycle

Use a **long-lived C++ network sidecar**.

```text
Tauri
  │
  │ spawn once
  ▼
C++ network sidecar
  │
  │ stdin: command
  │ stdout: response/snapshot
  │ stderr: diagnostics
  ▼
OS network provider
```

The sidecar should support a command protocol rather than being exclusively a one-shot CLI.

For example:

```text
Tauri → {"command":"network_snapshot"}
C++   → {"type":"network_snapshot", ...}

Tauri → {"command":"shutdown"}
C++   → {"type":"shutdown_ack"}
```

The existing disk scanner remains one-shot:

```bash
system-analyzer /path
```

Network monitoring becomes a long-lived mode:

```bash
system-analyzer --network
```

This is cleaner than spawning a C++ process every second and gives us a proper foundation for future monitoring functionality.

### 0.2 Persistence schema

Phase 1 creates **only interface-level persistence**:

```sql
network_rollups
```

We deliberately do **not** create:

```sql
app_usage_rollups
```

until Phase 6 establishes the application identity model.

That means the initial schema is stable:

```sql
CREATE TABLE network_rollups (
    ts INTEGER NOT NULL,
    interface_id TEXT NOT NULL,
    bytes_received INTEGER NOT NULL,
    bytes_sent INTEGER NOT NULL,
    PRIMARY KEY (ts, interface_id)
);
```

Then Phase 6 introduces the application table through a proper migration once we have settled `app_id`.

### 0.3 Monitoring semantics

For v1:

```text
Application running
        │
        ▼
Network monitoring active
        │
        ├── Network tab open
        │       └── live visualization
        │
        └── Network tab closed
                └── continue sampling
```

Closing the application stops monitoring.

Therefore:

> "24-hour history" means the last 24 hours of **observed application runtime**, not guaranteed continuous system-wide history.

The UI should explicitly represent gaps rather than interpolating data and pretending monitoring occurred.

Later:

```text
v2+
Background service / tray monitoring
        ↓
continuous telemetry
        ↓
true 24h system history
```

### 0.4 First vertical slice

After Phase 3 we immediately build:

```text
Linux
  ↓
Network provider
  ↓
Long-lived sidecar
  ↓
Tauri sampler
  ↓
Ring buffer
  ↓
Vue
```

Only:

* interface discovery
* RX/TX
* throughput
* live chart
* basic network summary

No application attribution yet.

This becomes our first **vertical integration gate**.

---

# Revised high-level sequence

```text
PHASE 0
Architecture decisions
    │
    ├── sidecar lifecycle
    ├── IPC protocol
    ├── monitoring semantics
    ├── sampling model
    └── persistence boundaries
    │
    ▼
PHASE 1
SQLite infrastructure
    │
    └── network_rollups only
    │
    ▼
PHASE 2
Linux network provider
    │
    ▼
PHASE 3
Sampling + ring buffer + rollups
    │
    ▼
PHASE 3.5
★ FIRST VERTICAL SLICE
    │
    ├── real C++ data
    ├── real Tauri IPC
    ├── real SQLite
    └── real Vue chart
    │
    ▼
PHASE 4
Windows provider
    │
    ▼
PHASE 5
macOS provider
    │
    ▼
PHASE 6
Application attribution
    │
    ├── identity model
    ├── app_usage_rollups migration
    ├── Linux
    ├── Windows
    └── macOS
    │
    ▼
PHASE 7
Application UI
    │
    ▼
PHASE 8
Historical analysis
    │
    ▼
PHASE 9
Performance + reliability
    │
    ▼
PHASE 10
Cross-platform integration
    │
    ▼
PHASE 11
Documentation + release
```

The next step is therefore **not implementation yet**. We should do **Phase 0.1 through 0.4**, inspect the existing Disk Analyzer sidecar architecture, and write the concrete Network Analyzer architecture/contract into `docs/DEVELOPMENT.md` before creating the first files.
