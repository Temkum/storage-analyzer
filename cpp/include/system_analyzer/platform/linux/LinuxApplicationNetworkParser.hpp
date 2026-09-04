#pragma once

#include "system_analyzer/domain/ApplicationNetworkUsage.hpp"

#include <cstdint>
#include <map>
#include <string>
#include <unordered_map>
#include <vector>

namespace system_analyzer
{

    // Deterministic, /proc-free helpers for application attribution. Exposed
    // so tests can feed fixture content rather than relying on a live machine;
    // the provider calls these against real /proc data. This mirrors the
    // LinuxNetworkParser test seam.

    struct TcpSocketRecord
    {
        std::uint32_t localAddressNum{0};
        std::uint16_t localPort{0};
        int state{0};
        std::uint64_t inode{0};
    };

    // Byte counters for a single socket inode (cumulative since boot).
    struct SocketBytes
    {
        std::uint64_t bytesReceived{0};
        std::uint64_t bytesSent{0};
    };

    // Parse /proc/net/tcp content into structured socket records.
    std::vector<TcpSocketRecord> parseApplicationTcpSockets(
        const std::string &content);

    // Aggregate per-app usage.
    //
    //   sockets   : parsed /proc/net/tcp records (only established counted)
    //   inodePid  : socket inode -> owning pid
    //   inodeBytes: socket inode -> cumulative byte counters
    //   exeOf     : pid -> canonical executable path ("" = skip)
    //   commOf    : pid -> display name
    //
    // Each socket is attributed to the process that owns its inode; sockets
    // whose inode -> pid -> exe chain is broken (unresolved, vanished,
    // inaccessible) are skipped. Byte counters for every socket owned by the
    // same executable are summed into a single ApplicationNetworkUsage whose
    // appId == executable path. Shared descriptors (same inode mapping to
    // multiple pids) are attributed once. Result is sorted by appId.
    std::vector<ApplicationNetworkUsage> aggregateApplicationUsage(
        const std::vector<TcpSocketRecord> &sockets,
        const std::unordered_map<std::uint64_t, int> &inodePid,
        const std::unordered_map<std::uint64_t, SocketBytes> &inodeBytes,
        const std::map<int, std::string> &exeOf,
        const std::map<int, std::string> &commOf);

} // namespace system_analyzer