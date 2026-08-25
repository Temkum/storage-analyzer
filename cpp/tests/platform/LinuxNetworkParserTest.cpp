#include <cassert>
#include <cstdint>
#include <string>
#include <vector>

#include "system_analyzer/platform/linux/LinuxNetworkParser.hpp"

using system_analyzer::NetworkInterface;
using system_analyzer::parseLinuxNetworkOperstate;
using system_analyzer::parseLinuxNetworkStats;

static const std::string kDevHeader =
    "Inter-|   Receive                                                |  Transmit\n"
    " face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n";

int main()
{
    // Normal content: multiple interfaces parsed correctly.
    {
        const std::string content = kDevHeader +
                                    "    lo: 2625297333  480141    0    0    0     0          0         0 "
                                    "2625297333  480141    0    0    0     0       0          0\n"
                                    "  eth0: 1843920312  12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n"
                                    "wlan0:  56789012  3456    0    0    0     0          0         0 "
                                    "45678901  2345    0    0    0     0       0          0\n";

        const auto interfaces = parseLinuxNetworkStats(content);

        assert(interfaces.size() == 3);

        assert(interfaces[0].id == "lo");
        assert(interfaces[0].name == "lo");
        assert(interfaces[0].bytesReceived == 2625297333);
        assert(interfaces[0].bytesSent == 2625297333);
        assert(!interfaces[0].isUp);

        assert(interfaces[1].id == "eth0");
        assert(interfaces[1].name == "eth0");
        assert(interfaces[1].bytesReceived == 1843920312);
        assert(interfaces[1].bytesSent == 92839122);
        assert(!interfaces[1].isUp);

        assert(interfaces[2].id == "wlan0");
        assert(interfaces[2].name == "wlan0");
        assert(interfaces[2].bytesReceived == 56789012);
        assert(interfaces[2].bytesSent == 45678901);
        assert(!interfaces[2].isUp);
    }

    // Domain contract: id equals name for every interface.
    {
        const std::string content = kDevHeader +
                                    "  eth0: 1843920312  12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n";

        const auto interfaces = parseLinuxNetworkStats(content);
        for (const auto &iface : interfaces)
        {
            assert(iface.id == iface.name);
        }
    }

    // Empty content (headers only, no data lines).
    {
        assert(parseLinuxNetworkStats(kDevHeader).empty());
    }

    // Completely empty input.
    {
        assert(parseLinuxNetworkStats("").empty());
    }

    // Malformed lines are skipped: no colon, non-numeric stats, too few fields.
    {
        const std::string content = kDevHeader +
                                    "  eth0: 1843920312  12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n"
                                    "this_line_has_no_colon\n"
                                    "  eth1: abc 12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n"
                                    "  eth2: 12345\n";

        const auto interfaces = parseLinuxNetworkStats(content);
        assert(interfaces.size() == 1);
        assert(interfaces[0].name == "eth0");
        assert(interfaces[0].bytesReceived == 1843920312);
        assert(interfaces[0].bytesSent == 92839122);
    }

    // Interface aliases (names containing ':') are filtered out.
    {
        const std::string content = kDevHeader +
                                    "  eth0: 1843920312  12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n"
                                    "eth0:0:  67890   100    0    0    0     0          0         0 "
                                    "11111   100    0    0    0     0       0          0\n";

        const auto interfaces = parseLinuxNetworkStats(content);
        assert(interfaces.size() == 1);
        assert(interfaces[0].name == "eth0");
        assert(interfaces[0].bytesReceived == 1843920312);
    }

    // Empty lines between data lines are skipped.
    {
        const std::string content = kDevHeader +
                                    "\n"
                                    "  eth0: 1843920312  12345    0    0    0     0          0         0 "
                                    "92839122  6789    0    0    0     0       0          0\n"
                                    "\n"
                                    "  eth1:    1000   50    0    0    0     0          0         0 "
                                    "2000   60    0    0    0     0       0          0\n";

        const auto interfaces = parseLinuxNetworkStats(content);
        assert(interfaces.size() == 2);
        assert(interfaces[0].name == "eth0");
        assert(interfaces[1].name == "eth1");
        assert(interfaces[1].bytesReceived == 1000);
        assert(interfaces[1].bytesSent == 2000);
    }

    // Long interface names parse correctly.
    {
        const std::string content = kDevHeader +
                                    "enx0c3796514e5b:       0       0    0    0    0     0          0         0 "
                                    "0       0    0    0    0     0       0          0\n";

        const auto interfaces = parseLinuxNetworkStats(content);
        assert(interfaces.size() == 1);
        assert(interfaces[0].name == "enx0c3796514e5b");
        assert(interfaces[0].bytesReceived == 0);
        assert(interfaces[0].bytesSent == 0);
    }

    // Operstate parsing: only "up" (trimmed) is considered up.
    {
        assert(parseLinuxNetworkOperstate("up\n") == true);
        assert(parseLinuxNetworkOperstate("down\n") == false);
        assert(parseLinuxNetworkOperstate("unknown\n") == false);
        assert(parseLinuxNetworkOperstate("up") == true);
        assert(parseLinuxNetworkOperstate("  up  \n") == true);
        assert(parseLinuxNetworkOperstate("") == false);
        assert(parseLinuxNetworkOperstate("\n") == false);
        assert(parseLinuxNetworkOperstate("dormant") == false);
    }

    return 0;
}