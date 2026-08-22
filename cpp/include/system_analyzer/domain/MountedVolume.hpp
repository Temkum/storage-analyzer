#pragma once

#include <cstdint>
#include <filesystem>
#include <string>

namespace system_analyzer::domain
{

    struct MountedVolume
    {
        std::filesystem::path mountPoint;
        std::string filesystem;
        std::uintmax_t totalBytes;
        std::uintmax_t freeBytes;
        std::uintmax_t availableBytes;
        std::uintmax_t usedBytes;
        bool readOnly = false;
    };

} // namespace system_analyzer::domain
