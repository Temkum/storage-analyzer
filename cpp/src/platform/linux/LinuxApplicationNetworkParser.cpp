#include "system_analyzer/platform/linux/LinuxApplicationNetworkParser.hpp"

#include <algorithm>
#include <cctype>
#include <sstream>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace system_analyzer
{

    namespace
    {
        std::string trim(const std::string &s)
        {
            const auto first = s.find_first_not_of(" \t\n\r");
            if (first == std::string::npos)
            {
                return "";
            }
            const auto last = s.find_last_not_of(" \t\n\r");
            return s.substr(first, last - first + 1);
        }

        // Parse a hex "ADDR:PORT" pair from /proc/net/tcp.
        bool parseAddress(const std::string &token,
                          std::uint32_t &addrNum,
                          std::uint16_t &port)
        {
            const auto colon = token.find(':');
            if (colon == std::string::npos)
            {
                return false;
            }

            const std::string addrPart = token.substr(0, colon);
            const std::string portPart = token.substr(colon + 1);

            if (addrPart.size() != 8 || portPart.size() != 4)
            {
                return false;
            }

            try
            {
                addrNum = static_cast<std::uint32_t>(
                    std::stoul(addrPart, nullptr, 16));
                port = static_cast<std::uint16_t>(
                    std::stoul(portPart, nullptr, 16));
            }
            catch (...)
            {
                return false;
            }

            return true;
        }
    } // namespace

    std::vector<TcpSocketRecord> parseApplicationTcpSockets(
        const std::string &content)
    {
        std::vector<TcpSocketRecord> entries;
        std::istringstream stream(content);
        std::string line;
        bool headerSkipped = false;

        while (std::getline(stream, line))
        {
            if (!headerSkipped)
            {
                headerSkipped = true;
                continue;
            }

            line = trim(line);
            if (line.empty())
            {
                continue;
            }

            std::istringstream ls(line);
            std::string sl, local, remote, st;
            ls >> sl >> local >> remote >> st;

            if (sl.empty() || local.empty() ||
                remote.empty() || st.empty())
            {
                continue;
            }

            TcpSocketRecord entry;

            if (!parseAddress(
                    local, entry.localAddressNum, entry.localPort))
            {
                continue;
            }

            try
            {
                entry.state = std::stoi(st, nullptr, 16);
            }
            catch (...)
            {
                continue;
            }

            // Remaining fields (real /proc/net/tcp merges tx_queue:rx_queue and
            // tr:tm->when into single tokens). After st, the tokens are:
            //   [0] txq:rxq  [1] tr:when  [2] retrnsmt  [3] uid
            //   [4] timeout  [5] inode  [6:] ref/sk/drops...
            std::string token;
            std::vector<std::string> remaining;
            while (ls >> token)
            {
                remaining.push_back(token);
            }

            if (remaining.size() < 6)
            {
                continue;
            }

            try
            {
                entry.inode = std::stoull(remaining[5], nullptr, 10);
            }
            catch (...)
            {
                continue;
            }

            entries.push_back(std::move(entry));
        }

        return entries;
    }

    std::vector<ApplicationNetworkUsage> aggregateApplicationUsage(
        const std::vector<TcpSocketRecord> &sockets,
        const std::unordered_map<std::uint64_t, int> &inodePid,
        const std::unordered_map<std::uint64_t, SocketBytes> &inodeBytes,
        const std::map<int, std::string> &exeOf,
        const std::map<int, std::string> &commOf)
    {
        // Per-executable accumulators.
        struct Accumulator
        {
            std::string processName;
            std::uint64_t bytesReceived = 0;
            std::uint64_t bytesSent = 0;
            int socketCount = 0;
        };

        std::unordered_map<std::string, Accumulator> byExePath;

        // Per-pid set of already-counted inodes to avoid double-counting
        // shared descriptors within one snapshot.
        std::unordered_map<int, std::unordered_set<std::uint64_t>> seenInodes;

        for (const auto &socket : sockets)
        {
            // Only established connections carry bytes.
            if (socket.state != 1) // TCP_ESTABLISHED
            {
                continue;
            }

            // Resolve inode -> pid.
            auto pidIt = inodePid.find(socket.inode);
            if (pidIt == inodePid.end())
            {
                continue; // unresolved inode
            }
            const int pid = pidIt->second;

            // Dedup shared descriptors.
            auto &seen = seenInodes[pid];
            if (seen.find(socket.inode) != seen.end())
            {
                continue;
            }
            seen.insert(socket.inode);

            // Resolve pid -> executable (identity). Empty => skipped.
            auto exeIt = exeOf.find(pid);
            if (exeIt == exeOf.end() || exeIt->second.empty())
            {
                continue;
            }
            const std::string &exePath = exeIt->second;

            // Optional display name.
            std::string processName;
            auto commIt = commOf.find(pid);
            if (commIt != commOf.end())
            {
                processName = commIt->second;
            }

            // Byte counters (default zero if the query failed).
            SocketBytes bytes;
            auto bytesIt = inodeBytes.find(socket.inode);
            if (bytesIt != inodeBytes.end())
            {
                bytes = bytesIt->second;
            }

            auto &acc = byExePath[exePath];
            if (acc.socketCount == 0)
            {
                acc.processName = processName;
            }
            acc.bytesReceived += bytes.bytesReceived;
            acc.bytesSent += bytes.bytesSent;
            acc.socketCount++;
        }

        std::vector<ApplicationNetworkUsage> result;
        result.reserve(byExePath.size());
        for (const auto &[exePath, acc] : byExePath)
        {
            result.push_back(
                {exePath, acc.processName, exePath,
                 acc.bytesReceived, acc.bytesSent});
        }

        std::sort(
            result.begin(),
            result.end(),
            [](const auto &a, const auto &b)
            {
                return a.appId < b.appId;
            });

        return result;
    }

} // namespace system_analyzer
