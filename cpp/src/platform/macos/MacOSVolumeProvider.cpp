#include "system_analyzer/platform/macos/MacOSVolumeProvider.hpp"

#include <sys/mount.h>

#include <cstring>
#include <string>
#include <unordered_set>
#include <vector>

namespace system_analyzer::platform::macos
{

    namespace
    {
        bool isPseudoFilesystem(const std::string &filesystem)
        {
            static const std::unordered_set<std::string> pseudoFilesystems = {
                "devfs",
                "autofs",
                "procfs",
                "fdesc",
                "devtmpfs"};

            return pseudoFilesystems.find(filesystem) !=
                   pseudoFilesystems.end();
        }
    }

    std::vector<domain::MountedVolume> MacOSVolumeProvider::getVolumes() const
    {
        std::vector<domain::MountedVolume> volumes;

        struct statfs *mounts = nullptr;

        const int count = getmntinfo(&mounts, MNT_WAIT);

        if (count <= 0 || mounts == nullptr)
        {
            return volumes;
        }

        for (int index = 0; index < count; ++index)
        {
            const struct statfs &entry = mounts[index];

            const std::string mountPoint(entry.f_mntonname);
            const std::string filesystem(entry.f_fstypename);

            if (isPseudoFilesystem(filesystem))
            {
                continue;
            }

            const auto blockSize =
                static_cast<std::uintmax_t>(entry.f_bsize);

            const auto totalBytes =
                static_cast<std::uintmax_t>(entry.f_blocks) *
                blockSize;

            const auto freeBytes =
                static_cast<std::uintmax_t>(entry.f_bfree) *
                blockSize;

            const auto availableBytes =
                static_cast<std::uintmax_t>(entry.f_bavail) *
                blockSize;

            const auto usedBytes =
                totalBytes > freeBytes
                    ? totalBytes - freeBytes
                    : 0;

            if (totalBytes == 0)
            {
                continue;
            }

            volumes.push_back({mountPoint,
                               filesystem,
                               totalBytes,
                               freeBytes,
                               availableBytes,
                               usedBytes,
                               (entry.f_flags & MNT_RDONLY) != 0});
        }

        return volumes;
    }

} // namespace system_analyzer::platform::macos