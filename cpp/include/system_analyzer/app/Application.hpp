#pragma once

#include <filesystem>

namespace system_analyzer::app
{

    class Application
    {
    public:
        int run(const std::filesystem::path &root);
    };

} // namespace system_analyzer::app
