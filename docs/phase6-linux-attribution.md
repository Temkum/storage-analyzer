# Phase 6 — Per-Application Network Attribution

## Status

**Investigation complete.** The first task — *"resolve the actual Linux mechanism
for obtaining per-process network byte counts"* — is resolved. See
[Decision: Mechanism](#decision-mechanism) below.

No code has been written yet. Per §6.1–6.5, we establish identity, prove attribution
accuracy, then (and only then) create `LinuxApplicationNetworkProvider` and the
`app_usage_rollups` migration.

## 1. Mechanism survey

Three mechanisms were evaluated. All were probed empirically on a real machine
(**Ubuntu 24.04, kernel 6.8.0-138-generic, uid 1000**). The read-only PoC source lives at
`docs/research/poc_tcpinfo.c`; run it with:

```sh
cc -O2 -o poc_tcpinfo docs/research/poc_tcpinfo.c && ./poc_tcpinfo
```

### 1.1 `/proc/net/tcp` + `/proc/<pid>/fd` + `/proc/<pid>/exe` (polling)

**Verdict: usable for same-uid (the user's own apps), blocked for foreign-uid.**

- `/proc/net/tcp` and `/proc/net/tcp6` are **world-readable** and contain per-socket
  UID columns. No privilege needed to enumerate sockets system-wide.
- `/proc/net/tcp` does **not** carry byte counters. Per-socket cumulative bytes come
  from `getsockopt(fd, IPPROTO_TCP, TCP_INFO, …)` — **but this requires an open file
  descriptor to the socket**. The sidecar has no such fd for another process's socket,
  so attribution must *open* `/proc/<pid>/fd/<n>` itself, which is gated by `/proc`
  access control.
- The inode→PID bridge (`/proc/<pid>/fd/* → socket:[inode]`) and the PID→exe bridge
  (`/proc/<pid>/exe`) are gated by Linux `/proc` access control: same UID or
  `CAP_SYS_PTRACE` → traversable; foreign UID without capabilities → `Permission denied`.

  ```
  $ ls /proc/1555/fd        # uid-133 process, our uid 1000
  ls: cannot open directory '/proc/1555/fd': Permission denied
  $ readlink /proc/1555/exe
                              # (empty — also denied)
  ```

  Note: `struct tcp_info` fields like `tcpi_bytes_sent` are **not** in glibc's outdated
  `netinet/tcp.h`; the provider must include the kernel UAPI header `<linux/tcp.h>`.

### 1.2 eBPF kprobes on `tcp_sendmsg` / `tcp_recvmsg` (tracing)

**Verdict: blocked for a standard unprivileged desktop install — but technically capable.**

- `kernel.unprivileged_bpf_disabled = 2` and our `CapEff: 0000000000000000` mean
  loading a kprobe program requires `CAP_BPF` / root. The desktop app runs as uid 1000
  with **no capabilities**, so eBPF is denied.
- The symbols exist (`tcp_sendmsg`, `tcp_recvmsg`, `udp_sendmsg`, `udpv6_sendmsg` are
  all exported in `/proc/kallsyms`), BTF is present at `/sys/kernel/btf/vmlinux`, and
  `clang-18` + `bpftrace` are installed — so eBPF is *technically* capable here, but
  only under root.
- eBPF is the **only** mechanism that can account for **UDP** byte counters and
  **foreign-uid** sockets accurately, because it instruments the send/recv path in
  kernel space rather than relying on per-process `/proc` visibility.

A privileged validation script is provided: `docs/research/poc_bpf_validate.sh`
(run `sudo bash docs/research/poc_bpf_validate.sh`). It traces the four send/recv
kprobes and confirms byte accounting with per-pid granularity, including UDP.

### 1.3 `/proc/<pid>/io` counters

**Verdict: not network-specific.** `rchar`/`wchar` in `/proc/<pid>/io` are
cumulative across **all** file descriptors — files, pipes, sockets, pty's. They cannot
isolate network bytes. Rejected.

## 2. Kernel evidence (run on this machine)

The PoC (`poc_tcpinfo.c`) produced, across 5/5 deterministic runs:

```
== F1: per-socket cumulative TCP counters (TCP_INFO) ==
CLIENT: bytes_sent=1000000 bytes_acked=1000001 bytes_received=500000 data_segs_out=23 data_segs_in=15
SERVER: bytes_sent=500000  bytes_acked=500000 bytes_received=1000000 data_segs_out=15 data_segs_in=31

== F2: identity chain /proc/net/tcp inode -> pid -> exe ==
  client local port 35170 -> socket inode 1166773
  inode -> pid: 282147 (expected 282147)
    exe: /tmp/phase6-poc/poc_tcpinfo

== F3: unresolvable inode is skipped, not fatal ==
  fake inode 424242424242 -> pid -1 (graceful skip: OK)

== F4: UDP has no cumulative byte counters ==
  udp row: 12598: 0100007F:81D1 07 ... 1171991 2 0000000000000000 0
  -> row shows queues/drops only, no cumulative bytes; socket found: yes
```

Key takeaways:

1. **TCP_INFO counters are exact to the octet.** The 1 000 000-byte payload appears
   verbatim as `bytes_sent` on the sender and `bytes_received` on the receiver.
2. **The inode→PID→exe chain resolves correctly** for the owning process (same uid).
3. **Unresolvable inodes are skipped, not fatal** — the Step 6.1 permission rule, proven.
4. **UDP sockets have no byte counters** in `/proc/net/udp` — only queues/uid/drops.

A system-wide socket walk confirmed the chain resolves same-uid sockets to real
executables:

```
sample own inode=1024329
  -> pid=252392 exe=/home/tem/.local/share/pnpm/.../cline comm=cline
```

The symlink test additionally confirmed `app_id` normalization:

```
argv/path: /tmp/phase6-poc/netan-symlink-sleep
comm:      netan-symlink-sleep     # truncated, invocation-derived
exe:       /usr/bin/sleep          # canonical, symlink-resolved
```

## 3. Decision: Mechanism

**Adopt the `/proc`-based polling model for Linux.** It gives **exact** attribution
for the dominant use case (the user's own apps) and degrades gracefully where the
kernel offers no data, rather than shipping an inaccurate counter.

| Traffic class       | Mechanism                  | Accuracy        | Notes                                  |
|---------------------|----------------------------|-----------------|----------------------------------------|
| TCP, same-uid       | `/proc/net/tcp` + TCP_INFO | **exact**       | Full attribution.                      |
| TCP, foreign-uid    | `/proc/net/tcp` + TCP_INFO | **not possible** as non-root | Socket is *visible* but the inode→pid→exe bridge is blocked by `/proc` perms (`EACCES`). |
| UDP (any)           | none in `/proc/net/udp`    | **not possible** via polling | No cumulative byte counters exist per-socket. |
| QUIC/HTTP3 (UDP)    | none                       | **not possible** via polling | Same UDP gap.                           |

**Implementation path for `LinuxApplicationNetworkProvider`:**

1. Snapshot `/proc/net/tcp` + `/proc/net/tcp6` → `{inode, uid, local_port, state}`.
2. Filter to **our uid** (`geteuid()`). Foreign-uid sockets are *visible* but
   *not attributable* — tally an "unattributed" bucket so the UI shows a gap, do **not** fail. *(Step 6.1 "skip rather than fail".)*
3. Resolve PID via `/proc/<pid>/fd/socket:[inode]`, then `app_id` via `/proc/<pid>/exe`
   (canonical, symlink-resolved). If `/proc/<pid>/exe` is unreadable → **skip that PID
   for this sample**, log to stderr, do not abort the cycle.
4. Open `/proc/<pid>/fd/<n>` + `getsockopt(fd, TCP_INFO)` for `tcpi_bytes_sent` /
   `tcpi_bytes_received` (cumulative since socket lifetime).
5. Accumulate per-PID cumulative counters; the existing Rust delta engine computes
   per-sample deltas; 60 s → `app_usage_rollups`.

**Why `/proc` over eBPF-first:** Phase 5 commits to a **non-root, always-on sidecar**.
eBPF requires `CAP_BPF`/root, which would force a privilege escalation across the
*entire* sidecar (including the interface telemetry path) — out of scope for v1. The
`/proc` path gives exact attribution for the dominant UI use case ("how much network
did my browser use this minute"). The UDP/foreign-uid gaps are **design constraints**
documented here; the `INetworkApplicationProvider` interface makes an eBPF backend
swappable in later. We do **not** build it now.

## 4. Application identity (Step 6.1 — locked)

```
app_id           = canonical executable path (readlink /proc/<pid>/exe, symlink-resolved)
process_name     = display name (`/proc/<pid>/comm`, best-effort, 15-char kernel limit)
executable_path  = same as app_id
```

- `/usr/lib/firefox/firefox` and a symlinked invocation resolve to the same `app_id`.
- PID is **never** used as `app_id`; restarts merge under the same `app_id`.
- If `/proc/<pid>/exe` is unreadable → **skip that PID** (Step 6.1 rule); logged, not fatal.
- `process_name` is best-effort UI decoration; `app_id` still stands if `comm` is unreadable.

## 5. Rust telemetry contract (Step 6.3 — interface)

Mirrors `apps/desktop/src-tauri/src/network/telemetry.rs` (`NetworkSample`):

```rust
pub struct ApplicationNetworkUsage {
    pub app_id: String,            // canonical executable path
    pub process_name: String,      // comm, best-effort
    pub executable_path: String,   // == app_id
    pub bytes_received: u64,       // cumulative since socket lifetime
    pub bytes_sent: u64,           // cumulative since socket lifetime
}
```

The Rust layer does **not** know these came from `/proc/net/tcp` + `TCP_INFO`. It only
sees `ApplicationSnapshot` (`ts` + `Vec<ApplicationNetworkUsage>`). Same delta /
ring-buffer / 60 s rollup pipeline, keyed by `app_id`.

NDJSON contract: the existing `network_snapshot` message gains an `applications` array
(the C++ provider supplies it; Rust parses it — keeping the platform boundary):

```jsonc
{
  "type": "network_snapshot", "ts": 1724000000,
  "interfaces": [{"id":"eth0","name":"eth0","bytesReceived":123,"bytesSent":456,"isUp":true}],
  "applications": [{"appId":"/usr/lib/firefox/firefox","processName":"Web Content",
    "executablePath":"/usr/lib/firefox/firefox","bytesReceived":1420000,"bytesSent":850000}]
}
```

## 6. Schema (Step 6.4 — NOT created yet)

Existing `schema_migrations` (`CURRENT_SCHEMA_VERSION = 1`,
`storage/migrations.rs`). `app_usage_rollups` stays deferred. Target shape (migration #2):

```sql
CREATE TABLE app_usage_rollups (
    ts              INTEGER NOT NULL,
    app_id          TEXT    NOT NULL,
    process_name    TEXT    NOT NULL,
    executable_path TEXT,
    bytes_received  INTEGER NOT NULL,
    bytes_sent      INTEGER NOT NULL,
    PRIMARY KEY     (ts, app_id)
);
CREATE INDEX idx_app_usage_rollups_ts ON app_usage_rollups(ts);
```

## 7. Remaining Phase 6 gates

- [x] Application identity · [x] Linux process enumeration · [x] Executable identification
- [x] Network ownership/accounting · [x] Permission failures tolerated · [x] No PID as identity
- [ ] Multiple applications · [ ] Process restart handling · [ ] Short-lived process handling
- [ ] Rust application telemetry model (contract defined §5; not coded) · [ ] Rollup schema (deferred §6)
- [ ] Deterministic tests · [ ] Real-machine integration test (PoC is the kernel-truth baseline)

## 8. Next action

1. Create `INetworkApplicationProvider` C++ interface + `ApplicationNetworkUsage` /
   `ApplicationSnapshot` domain types (mirroring `NetworkSnapshot`).
2. Implement `LinuxApplicationNetworkProvider` per §3 — same-uid TCP only, graceful skip
   on every failure, UDP gap surfaced as `app_id = "__unattributed_udp__"`.
3. Add Rust `ApplicationNetworkUsage` type + delta/rollup mirroring `telemetry.rs`.
4. Deterministic tests: counter-exact attribution, fake-inode skip, symlink→canonical-exe,
   UDP "no counter" path.
5. **Then** migration #2 (`app_usage_rollups`) + bump schema version to 2.

Privileged eBPF validation: `docs/research/poc_bpf_validate.sh` (run
`sudo bash docs/research/poc_bpf_validate.sh`) — out of scope for v1.
