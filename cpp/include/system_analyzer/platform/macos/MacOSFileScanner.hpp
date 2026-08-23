#pragma once

#include <filesystem>

#include "system_analyzer/core/IFileScanner.hpp"
#include "system_analyzer/platform/common/StandardFileScanner.hpp"

namespace system_analyzer::platform::macos
{

    /// macOS file scanner; traversal logic is shared via the portable
    /// standard scanner (std::filesystem based).
    class MacOSFileScanner final : public common::StandardFileScanner
    {
    };

} // namespace system_analyzer::platform::macos