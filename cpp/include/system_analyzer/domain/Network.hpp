#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace system_analyzer
{

    struct NetworkInterface
    {
        std::string id;
        std::string name;
        std::uint64_t bytesReceived;
        std::uint64_t bytesSent;
        bool isUp;
    };

    struct NetworkSnapshot
    {
        std::uint64_t timestamp;
        std::vector<NetworkInterface> interfaces;
    };

} // namespace system_analyzer