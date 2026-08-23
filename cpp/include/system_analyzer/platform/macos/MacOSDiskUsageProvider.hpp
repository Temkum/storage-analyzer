#pragma once

#include <filesystem>

#include "system_analyzer/core/IDiskUsageProvider.hpp"

namespace system_analyzer::platform::macos
{

    /// Disk usage for the given path via statvfs (POSIX, available on macOS).
    class MacOSDiskUsageProvider final : public core::IDiskUsageProvider
    {
    public:
        [[nodiscard]] domain::DiskUsage getUsage(
            const std::filesystem::path &path) const override;
    };

} // namespace system_analyzer::platform::macos