#pragma once

#include "system_analyzer/core/IFileScanner.hpp"

namespace system_analyzer::platform::linux {

class LinuxFileScanner final : public core::IFileScanner {
public:
    void scan(
        const std::filesystem::path& root,
        const EntryCallback& callback
    ) override;
};

} // namespace system_analyzer::platform::linux
