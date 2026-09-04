#pragma once

#include <string>

#include "system_analyzer/domain/ApplicationNetworkUsage.hpp"

namespace system_analyzer::serialization
{

class ApplicationNetworkSnapshotSerializer
{
public:
    [[nodiscard]] static std::string toJson(
        const ApplicationNetworkSnapshot &snapshot);
};

} // namespace system_analyzer::serialization