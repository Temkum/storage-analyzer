#include "system_analyzer/platform/factory.hpp"

#include "system_analyzer/platform/macos/MacOSDiskUsageProvider.hpp"
#include "system_analyzer/platform/macos/MacOSFileScanner.hpp"
#include "system_analyzer/platform/macos/MacOSVolumeProvider.hpp"

namespace system_analyzer::platform
{

    std::unique_ptr<core::IFileScanner> createFileScanner()
    {
        return std::make_unique<macos::MacOSFileScanner>();
    }

    std::unique_ptr<core::IDiskUsageProvider> createDiskUsageProvider()
    {
        return std::make_unique<macos::MacOSDiskUsageProvider>();
    }

    std::unique_ptr<core::IVolumeProvider> createVolumeProvider()
    {
        return std::make_unique<macos::MacOSVolumeProvider>();
    }

    // Application attribution is not yet implemented on macOS (Phase 6 is
    // Linux-first). Return nullptr so callers can detect absence gracefully.
    std::unique_ptr<IApplicationNetworkProvider>
    createApplicationNetworkProvider()
    {
        return nullptr;
    }

} // namespace system_analyzer::platform