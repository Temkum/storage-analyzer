#include <cassert>
#include <filesystem>

#include "system_analyzer/platform/linux/LinuxVolumeProvider.hpp"

int main()
{
    using system_analyzer::platform::linux::LinuxVolumeProvider;

    LinuxVolumeProvider provider;

    const auto volumes = provider.getVolumes();

    assert(!volumes.empty());

    for (const auto &volume : volumes)
    {
        assert(!volume.mountPoint.empty());
        assert(!volume.filesystem.empty());

        assert(volume.totalBytes > 0);
        assert(volume.freeBytes <= volume.totalBytes);
        assert(volume.availableBytes <= volume.totalBytes);
        assert(volume.usedBytes <= volume.totalBytes);
    }

    return 0;
}
