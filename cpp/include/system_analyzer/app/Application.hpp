#pragma once

#include <filesystem>

#include "system_analyzer/core/ScanContext.hpp"
#include "system_analyzer/domain/ScanResult.hpp"

namespace system_analyzer::app
{

    class Application
    {
    public:
        [[nodiscard]] domain::ScanResult scan(
            const std::filesystem::path &root,
            const core::ScanContext &context = {});

        /// Long-lived NDJSON sidecar loop for network monitoring: reads one
        /// JSON command per line from stdin and writes one JSON response per
        /// line to stdout until shutdown is requested or stdin closes.
        [[nodiscard]] int runNetworkMode();
    };

} // namespace system_analyzer::app