#pragma once

#include <cstdint>
#include <filesystem>
#include <vector>

#include "system_analyzer/domain/DiskUsage.hpp"
#include "system_analyzer/domain/FileEntry.hpp"
#include "system_analyzer/domain/MountedVolume.hpp"
#include "system_analyzer/domain/ScanError.hpp"

namespace system_analyzer::domain
{

    struct DirectorySize
    {
        std::filesystem::path path;
        std::uintmax_t size;
    };

    struct ScanResult
    {
        std::filesystem::path rootPath;

        std::uintmax_t totalSize = 0;

        std::uintmax_t fileCount = 0;

        std::uintmax_t directoryCount = 0;

        std::uint64_t durationMs = 0;

        DiskUsage diskUsage;

        std::vector<MountedVolume> volumes;

        std::vector<FileEntry> entries;

        std::vector<DirectorySize> directories;

        std::vector<ScanError> errors;
    };

} // namespace system_analyzer::domain