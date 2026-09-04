#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace system_analyzer
{

    /// One application's cumulative network usage since boot.
    ///
    /// Identity is the canonical executable path (appId == executablePath),
    /// never the PID. A PID is ephemeral across restarts; the executable path
    /// is stable. processName is a display-only label from /proc/<pid>/comm.
    struct ApplicationNetworkUsage
    {
        std::string appId;
        std::string processName;
        std::string executablePath;
        std::uint64_t bytesReceived;
        std::uint64_t bytesSent;
    };

    /// Cumulative snapshot of per-application network usage at a point in time.
    /// The provider reports cumulative counters; Rust computes deltas.
    struct ApplicationNetworkSnapshot
    {
        std::uint64_t timestamp;
        std::vector<ApplicationNetworkUsage> applications;
    };

} // namespace system_analyzer