#pragma once

#include "system_analyzer/platform/application_network_provider.hpp"

namespace system_analyzer
{

    /// Linux implementation of IApplicationNetworkProvider.
    ///
    /// Attribution pipeline:
    ///   /proc/net/tcp → socket inode → enumerate /proc/<pid>/fd →
    ///   socket:[inode] → PID → /proc/<pid>/exe → canonical executable →
    ///   SOCK_DIAG (INET_DIAG_INFO / tcp_info) byte counters → aggregate by exe
    ///
    /// Per-socket cumulative byte counters come from a SOCK_DIAG netlink dump
    /// (SOCK_DIAG_BY_FAMILY, reading INET_DIAG_INFO) — the same mechanism
    /// `ss -eit` uses to read another process's TCP_INFO. It works
    /// unprivileged for same-uid sockets.
    ///
    /// Identity: canonical executable path (from readlink /proc/<pid>/exe).
    /// PID is used only for per-socket dedup within a single snapshot, never
    /// as historical identity.
    ///
    /// Permission tolerance: EACCES on /proc/<pid>/fd or EPERM on
    /// /proc/<pid>/exe skips that process. A socket disappearing between
    /// /proc/net/tcp and the byte-counter query is skipped. One
    /// inaccessible or vanishing process never fails the whole snapshot.
    class LinuxApplicationNetworkProvider final
        : public IApplicationNetworkProvider
    {
    public:
        ApplicationNetworkSnapshot getSnapshot() override;
    };

} // namespace system_analyzer