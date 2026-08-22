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
    };

} // namespace system_analyzer::app