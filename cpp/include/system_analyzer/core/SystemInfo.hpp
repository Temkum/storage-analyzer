#pragma once

#include <string>

namespace system_analyzer::core
{

    class SystemInfo
    {
    public:
        explicit SystemInfo(std::string name);

        [[nodiscard]] const std::string &name() const;

    private:
        std::string name_;
    };

} // namespace system_analyzer::core
