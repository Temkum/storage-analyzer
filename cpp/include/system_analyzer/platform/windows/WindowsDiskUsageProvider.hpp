#pragma once

#include <filesystem>

#include "system_analyzer/core/IDiskUsageProvider.hpp"

namespace system_analyzer::platform::windows
{

    /// Disk usage for the drive that contains the given path, via
    /// GetDiskFreeSpaceExW.
    class WindowsDiskUsageProvider final : public core::IDiskUsageProvider
    {
    public:
        [[nodiscard]] domain::DiskUsage getUsage(
            const std::filesystem::path &path) const override;
    };

} // namespace system_analyzer::platform::windows