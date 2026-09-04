#pragma once

#include "system_analyzer/domain/Network.hpp"

#include <string>
#include <vector>

namespace system_analyzer
{

    std::vector<NetworkInterface> parseLinuxNetworkStats(
        const std::string &contents);

    bool parseLinuxNetworkOperstate(
        const std::string &contents);

} // namespace system_analyzer
