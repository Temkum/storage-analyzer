#pragma once

#include <filesystem>

#include "system_analyzer/domain/FileEntry.hpp"

namespace system_analyzer::platform::linux {

class LinuxFileEntryMapper {
public:
    static domain::FileEntry map(
        const std::filesystem::directory_entry& entry
    );
};

} // namespace system_analyzer::platform::linux
