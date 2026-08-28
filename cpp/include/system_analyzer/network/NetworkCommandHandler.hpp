#pragma once

#include "system_analyzer/platform/network_usage_provider.hpp"

#include <string>

namespace system_analyzer
{

    class NetworkCommandHandler
    {
    public:
        explicit NetworkCommandHandler(INetworkUsageProvider &provider);

        std::string handle(const std::string &request);

    private:
        INetworkUsageProvider &provider;
    };

} // namespace system_analyzer