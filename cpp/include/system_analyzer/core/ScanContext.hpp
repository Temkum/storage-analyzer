#pragma once

#include <cstdint>
#include <functional>

namespace system_analyzer::core
{

    struct ScanContext
    {
        std::function<void(std::uintmax_t)> onProgress;
        std::function<bool()> isCancelled;
    };

} // namespace system_analyzer::core