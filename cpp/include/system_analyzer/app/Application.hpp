#pragma once

#include <filesystem>

#include "system_analyzer/domain/ScanResult.hpp"

namespace system_analyzer::app
{

    class Application
    {
    public:
        [[nodiscard]] domain::ScanResult scan(
            const std::filesystem::path &root);
    };

} // namespace system_analyzer::app