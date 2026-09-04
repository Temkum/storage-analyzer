#include "system_analyzer/platform/linux/LinuxNetworkUsageProvider.hpp"

#include <cstdint>
#include <ctime>
#include <fstream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

#include "system_analyzer/platform/linux/LinuxNetworkParser.hpp"

namespace system_analyzer
{

    NetworkSnapshot LinuxNetworkUsageProvider::getSnapshot() const
    {
        std::ifstream dev("/proc/net/dev");
        if (!dev)
        {
            throw std::runtime_error("Failed to open /proc/net/dev");
        }

        std::stringstream ss;
        ss << dev.rdbuf();
        std::string content = ss.str();

        std::vector<NetworkInterface> interfaces =
            parseLinuxNetworkStats(content);

        for (auto &iface : interfaces)
        {
            std::ifstream operstate(
                "/sys/class/net/" + iface.name + "/operstate");

            if (operstate)
            {
                std::string stateContent;
                std::getline(operstate, stateContent);
                iface.isUp = parseLinuxNetworkOperstate(stateContent);
            }
        }

        return NetworkSnapshot{
            static_cast<std::uint64_t>(std::time(nullptr)),
            std::move(interfaces)};
    }

} // namespace system_analyzer