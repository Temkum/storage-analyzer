#pragma once

#include <memory>

#include "system_analyzer/core/IDiskUsageProvider.hpp"
#include "system_analyzer/core/IFileScanner.hpp"
#include "system_analyzer/core/IVolumeProvider.hpp"

namespace system_analyzer::platform
{

    /// Platform selection happens inside this factory (implemented once per
    /// platform under cpp/src/platform/<os>/PlatformFactory.cpp); application
    /// code never branches on the operating system.
    [[nodiscard]] std::unique_ptr<core::IFileScanner> createFileScanner();

    [[nodiscard]] std::unique_ptr<core::IDiskUsageProvider> createDiskUsageProvider();

    [[nodiscard]] std::unique_ptr<core::IVolumeProvider> createVolumeProvider();

} // namespace system_analyzer::platform