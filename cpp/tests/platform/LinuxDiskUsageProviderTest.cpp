#include <cassert>
#include <cstdint>
#include <filesystem>
#include <stdexcept>

#include "system_analyzer/platform/linux/LinuxDiskUsageProvider.hpp"

using system_analyzer::domain::DiskUsage;
using system_analyzer::platform::linux::LinuxDiskUsageProvider;

int main()
{
    const LinuxDiskUsageProvider provider;

    const std::filesystem::path tempRoot =
        std::filesystem::temp_directory_path() / "linux-disk-usage-test";

    std::filesystem::create_directories(tempRoot);

    const DiskUsage usage = provider.getUsage(tempRoot);

    std::filesystem::remove_all(tempRoot);

    // The file system fragment size (f_frsize) is 4096 on virtually all
    // Linux systems, which makes byte math deterministic.
    assert(usage.path == tempRoot);

    assert(usage.totalBytes > 0);
    assert(usage.totalBytes % 4096 == 0);

    assert(usage.freeBytes >= 0);
    assert(usage.freeBytes <= usage.totalBytes);

    assert(usage.availableBytes >= 0);
    assert(usage.availableBytes <= usage.freeBytes);

    assert(usage.usedBytes >= 0);
    assert(usage.usedBytes <= usage.totalBytes);

    assert(usage.usedBytes == usage.totalBytes - usage.freeBytes);

    bool threw = false;

    try
    {
        static_cast<void>(provider.getUsage(
            "/system-analyzer-path-that-does-not-exist"));
    }
    catch (const std::runtime_error &)
    {
        threw = true;
    }

    assert(threw);

    return 0;
}
