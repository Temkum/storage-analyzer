#pragma once

#include <filesystem>

#include "system_analyzer/domain/DiskUsage.hpp"

namespace system_analyzer::core
{

    class IDiskUsageProvider
    {
    public:
        virtual ~IDiskUsageProvider() = default;

        [[nodiscard]] virtual domain::DiskUsage getUsage(
            const std::filesystem::path &path) const = 0;
    };

} // namespace system_analyzer::core
