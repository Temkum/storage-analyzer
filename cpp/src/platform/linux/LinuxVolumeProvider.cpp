#include "system_analyzer/platform/linux/LinuxVolumeProvider.hpp"

#include <cerrno>
#include <cstring>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <sys/statvfs.h>
#include <unordered_set>

namespace system_analyzer::platform::linux
{

    namespace
    {
        bool isPseudoFilesystem(const std::string &filesystem)
        {
            static const std::unordered_set<std::string> pseudoFilesystems = {
                "proc",
                "sysfs",
                "devtmpfs",
                "devpts",
                "cgroup",
                "cgroup2",
                "pstore",
                "debugfs",
                "tracefs",
                "securityfs",
                "configfs",
                "fusectl",
                "mqueue",
                "hugetlbfs",
                "bpf",
                "ramfs",
                "overlay",
                "autofs",
                "binfmt_misc",
                "nsfs"};

            return pseudoFilesystems.contains(filesystem);
        }

        std::string unescapeMountField(const std::string &value)
        {
            std::string result;
            result.reserve(value.size());

            for (std::size_t i = 0; i < value.size(); ++i)
            {
                if (value[i] == '\\' && i + 3 < value.size())
                {
                    const std::string escaped = value.substr(i, 4);

                    if (escaped == "\\040")
                    {
                        result += ' ';
                        i += 3;
                        continue;
                    }

                    if (escaped == "\\011")
                    {
                        result += '\t';
                        i += 3;
                        continue;
                    }

                    if (escaped == "\\134")
                    {
                        result += '\\';
                        i += 3;
                        continue;
                    }
                }

                result += value[i];
            }

            return result;
        }
    }

    std::vector<domain::MountedVolume>
    LinuxVolumeProvider::getVolumes() const
    {
        std::ifstream mounts("/proc/self/mounts");

        if (!mounts)
        {
            throw std::runtime_error(
                "Failed to open /proc/self/mounts");
        }

        std::vector<domain::MountedVolume> volumes;

        std::string line;

        while (std::getline(mounts, line))
        {
            std::istringstream stream(line);

            std::string device;
            std::string mountPoint;
            std::string filesystem;

            if (!(stream >> device >> mountPoint >> filesystem))
            {
                continue;
            }

            mountPoint = unescapeMountField(mountPoint);
            filesystem = unescapeMountField(filesystem);

            if (isPseudoFilesystem(filesystem))
            {
                continue;
            }

            struct statvfs stats{};

            if (statvfs(mountPoint.c_str(), &stats) != 0)
            {
                continue;
            }

            const auto blockSize =
                static_cast<std::uintmax_t>(stats.f_frsize);

            const auto totalBytes =
                static_cast<std::uintmax_t>(stats.f_blocks) *
                blockSize;

            const auto freeBytes =
                static_cast<std::uintmax_t>(stats.f_bfree) *
                blockSize;

            const auto availableBytes =
                static_cast<std::uintmax_t>(stats.f_bavail) *
                blockSize;

            const auto usedBytes =
                totalBytes > freeBytes
                    ? totalBytes - freeBytes
                    : 0;

            // A volume that reports zero capacity (e.g. namespace inodes,
            // autofs placeholders, or FUSE bridges) is not a real mounted
            // filesystem, so it is excluded from the result.
            if (totalBytes == 0)
            {
                continue;
            }

            volumes.push_back({mountPoint,
                               filesystem,
                               totalBytes,
                               freeBytes,
                               availableBytes,
                               usedBytes});
        }

        return volumes;
    }

} // namespace system_analyzer::platform::linux