#pragma once

#include "system_analyzer/platform/application_network_provider.hpp"
#include "system_analyzer/platform/network_usage_provider.hpp"

#include <string>

namespace system_analyzer
{

    class NetworkCommandHandler
    {
    public:
        NetworkCommandHandler(
            INetworkUsageProvider &provider,
            IApplicationNetworkProvider &appProvider);

        std::string handle(const std::string &request);

    private:
        INetworkUsageProvider &provider;
        IApplicationNetworkProvider &appProvider;
    };

} // namespace system_analyzer