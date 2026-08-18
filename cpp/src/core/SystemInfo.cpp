#include "system_analyzer/core/SystemInfo.hpp"

#include <utility>

namespace system_analyzer::core
{

    SystemInfo::SystemInfo(std::string name)
        : name_(std::move(name))
    {
    }

    const std::string &SystemInfo::name() const
    {
        return name_;
    }

} // namespace system_analyzer::core
