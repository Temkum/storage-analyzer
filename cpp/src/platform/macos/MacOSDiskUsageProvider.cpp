#include "system_analyzer/platform/macos/MacOSDiskUsageProvider.hpp"

#include <cerrno>
#include <cstdint>
#include <cstring>
#include <stdexcept>
#include <string>
#include <sys/statvfs.h>

namespace system_analyzer::platform::macos
{

    domain::DiskUsage MacOSDiskUsageProvider::getUsage(
        const std::filesystem::path &path) const
    {
        struct statvfs stats{};

        if (statvfs(path.c_str(), &stats) != 0)
        {
            throw std::runtime_error(
                "Failed to get filesystem statistics for \"" +
                path.string() + "\": " +
                std::string(std::strerror(errno)));
        }

        const auto totalBytes =
            static_cast<std::uintmax_t>(stats.f_blocks) *
            static_cast<std::uintmax_t>(stats.f_frsize);

        const auto freeBytes =
            static_cast<std::uintmax_t>(stats.f_bfree) *
            static_cast<std::uintmax_t>(stats.f_frsize);

        const auto availableBytes =
            static_cast<std::uintmax_t>(stats.f_bavail) *
            static_cast<std::uintmax_t>(stats.f_frsize);

        const auto usedBytes =
            totalBytes > freeBytes
                ? totalBytes - freeBytes
                : 0;

        return {
            path,
            totalBytes,
            freeBytes,
            availableBytes,
            usedBytes};
    }

} // namespace system_analyzer::platform::macos