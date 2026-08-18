#pragma once

#include <cstdint>
#include <filesystem>

namespace system_analyzer::domain
{

    enum class FileType
    {
        File,
        Directory,
        Symlink,
        Other
    };

    struct FileEntry
    {
        std::filesystem::path path;
        FileType type;
        std::uintmax_t size;
    };

} // namespace system_analyzer::domain
