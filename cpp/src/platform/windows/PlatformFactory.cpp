#include "system_analyzer/platform/factory.hpp"

#include "system_analyzer/platform/windows/WindowsDiskUsageProvider.hpp"
#include "system_analyzer/platform/windows/WindowsFileScanner.hpp"
#include "system_analyzer/platform/windows/WindowsVolumeProvider.hpp"

namespace system_analyzer::platform
{

    std::unique_ptr<core::IFileScanner> createFileScanner()
    {
        return std::make_unique<windows::WindowsFileScanner>();
    }

    std::unique_ptr<core::IDiskUsageProvider> createDiskUsageProvider()
    {
        return std::make_unique<windows::WindowsDiskUsageProvider>();
    }

    std::unique_ptr<core::IVolumeProvider> createVolumeProvider()
    {
        return std::make_unique<windows::WindowsVolumeProvider>();
    }

} // namespace system_analyzer::platform