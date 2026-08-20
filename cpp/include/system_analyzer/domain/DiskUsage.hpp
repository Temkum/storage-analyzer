#pragma once

#include <cstdint>
#include <filesystem>

namespace system_analyzer::domain
{

    struct DiskUsage
    {
        std::filesystem::path path;
        std::uintmax_t totalBytes;
        std::uintmax_t freeBytes;
        std::uintmax_t availableBytes;
        std::uintmax_t usedBytes;
    };

} // namespace system_analyzer::domain
