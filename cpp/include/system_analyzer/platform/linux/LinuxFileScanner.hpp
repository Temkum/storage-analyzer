#pragma once

#include "system_analyzer/platform/common/StandardFileScanner.hpp"

namespace system_analyzer::platform::linux
{

    /// Linux file scanner; traversal logic is shared via the portable
    /// standard scanner (std::filesystem based).
    class LinuxFileScanner final : public common::StandardFileScanner
    {
    };

} // namespace system_analyzer::platform::linux
