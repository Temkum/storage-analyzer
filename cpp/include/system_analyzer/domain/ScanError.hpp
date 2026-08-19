#pragma once

#include <string>

namespace system_analyzer::domain
{

    struct ScanError
    {
        std::string path;
        std::string message;
    };

} // namespace system_analyzer::domain
