#include "system_analyzer/platform/factory.hpp"

#include "system_analyzer/platform/linux/LinuxApplicationNetworkProvider.hpp"
#include "system_analyzer/platform/linux/LinuxDiskUsageProvider.hpp"
#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"
#include "system_analyzer/platform/linux/LinuxNetworkUsageProvider.hpp"
#include "system_analyzer/platform/linux/LinuxVolumeProvider.hpp"

namespace system_analyzer::platform
{

    std::unique_ptr<core::IFileScanner> createFileScanner()
    {
        return std::make_unique<linux::LinuxFileScanner>();
    }

    std::unique_ptr<core::IDiskUsageProvider> createDiskUsageProvider()
    {
        return std::make_unique<linux::LinuxDiskUsageProvider>();
    }

    std::unique_ptr<core::IVolumeProvider> createVolumeProvider()
    {
        return std::make_unique<linux::LinuxVolumeProvider>();
    }

    std::unique_ptr<INetworkUsageProvider> createNetworkUsageProvider()
    {
        return std::make_unique<LinuxNetworkUsageProvider>();
    }

    std::unique_ptr<IApplicationNetworkProvider>
    createApplicationNetworkProvider()
    {
        return std::make_unique<LinuxApplicationNetworkProvider>();
    }

} // namespace system_analyzer::platform