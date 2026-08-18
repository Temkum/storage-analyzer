#pragma once

#include <cstdint>
#include <filesystem>
#include <vector>

#include "system_analyzer/domain/FileEntry.hpp"

namespace system_analyzer::domain {

struct DirectorySize {
    std::filesystem::path path;
    std::uintmax_t size;
};

struct ScanResult {
    std::vector<FileEntry> entries;
    std::vector<DirectorySize> directories;
};

} // namespace system_analyzer::domain
