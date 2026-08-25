#pragma once

#include "system_analyzer/domain/Network.hpp"

#include <string>

namespace system_analyzer
{

    class INetworkUsageProvider
    {
    public:
        virtual ~INetworkUsageProvider() = default;

        virtual NetworkSnapshot getSnapshot() const = 0;
    };

} // namespace system_analyzer