#pragma once

#include <filesystem>

#include "system_analyzer/core/IDiskUsageProvider.hpp"

namespace system_analyzer::platform::linux
{

    class LinuxDiskUsageProvider final
        : public core::IDiskUsageProvider
    {
    public:
        [[nodiscard]] domain::DiskUsage getUsage(
            const std::filesystem::path &path) const override;
    };

} // namespace system_analyzer::platform::linux
