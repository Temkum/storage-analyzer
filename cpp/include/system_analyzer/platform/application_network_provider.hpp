#pragma once

#include "system_analyzer/domain/ApplicationNetworkUsage.hpp"

#include <vector>

namespace system_analyzer
{

    /// Platform-neutral interface for per-application network attribution.
    ///
    /// Implementations report cumulative byte counters (like the interface
    /// provider). Rust remains responsible for delta calculation, ring buffer
    /// and rollup. The telemetry layer never knows whether the data came from
    /// /proc, ETW, Network Extension APIs, or another OS mechanism.
    class IApplicationNetworkProvider
    {
    public:
        virtual ~IApplicationNetworkProvider() = default;

        /// Returns cumulative per-application network usage since boot.
        /// One inaccessible or disappearing process must never make the entire
        /// snapshot fail: skip it and continue.
        virtual ApplicationNetworkSnapshot getSnapshot() = 0;
    };

} // namespace system_analyzer