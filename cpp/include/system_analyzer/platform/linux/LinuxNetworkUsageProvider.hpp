#pragma once

#include "system_analyzer/platform/network_usage_provider.hpp"

namespace system_analyzer
{

    class LinuxNetworkUsageProvider final : public INetworkUsageProvider
    {
    public:
        NetworkSnapshot getSnapshot() const override;
    };

} // namespace system_analyzer