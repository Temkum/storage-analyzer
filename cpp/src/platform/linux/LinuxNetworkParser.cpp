#include "system_analyzer/platform/linux/LinuxNetworkParser.hpp"

#include <cstdint>
#include <sstream>
#include <string>
#include <vector>

namespace system_analyzer
{

    namespace
    {

        // Trim leading and trailing whitespace characters.
        std::string trimWhitespace(const std::string &s)
        {
            const auto first = s.find_first_not_of(" \t\n\r");
            if (first == std::string::npos)
            {
                return "";
            }
            const auto last = s.find_last_not_of(" \t\n\r");
            return s.substr(first, last - first + 1);
        }

    } // namespace

    std::vector<NetworkInterface> parseLinuxNetworkStats(const std::string &content)
    {
        std::vector<NetworkInterface> interfaces;
        std::istringstream stream(content);
        std::string line;

        int lineNumber = 0;
        while (std::getline(stream, line))
        {
            ++lineNumber;

            // Skip the two header lines.
            if (lineNumber <= 2)
            {
                continue;
            }

            // Find the last ':' separating the interface name from the stats.
            // rfind handles aliases like "eth0:0:" correctly.
            const auto colonPos = line.rfind(':');
            if (colonPos == std::string::npos)
            {
                continue; // malformed: no colon delimiter
            }

            std::string name = trimWhitespace(line.substr(0, colonPos));
            if (name.empty())
            {
                continue; // malformed: empty interface name
            }

            // Skip interface aliases (names containing ':').
            if (name.find(':') != std::string::npos)
            {
                continue;
            }

            // Parse the 16 numeric fields after the colon.
            std::istringstream stats(line.substr(colonPos + 1));
            std::vector<std::uint64_t> values;
            std::string field;

            while (stats >> field)
            {
                try
                {
                    values.push_back(std::stoull(field));
                }
                catch (...)
                {
                    values.clear();
                    break;
                }
            }

            // /proc/net/dev emits 16 fields; we need at least 9
            // (up to tx_bytes at index 8).
            if (values.size() < 9)
            {
                continue; // malformed: too few fields
            }

            interfaces.push_back({name, name, values[0], values[8], false});
        }

        return interfaces;
    }

    bool parseLinuxNetworkOperstate(const std::string &content)
    {
        return trimWhitespace(content) == "up";
    }

} // namespace system_analyzer
