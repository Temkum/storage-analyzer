#include <cassert>
#include <iostream>
#include <map>
#include <string>
#include <unordered_map>
#include <vector>

#include "system_analyzer/platform/linux/LinuxApplicationNetworkParser.hpp"

using system_analyzer::aggregateApplicationUsage;
using system_analyzer::ApplicationNetworkUsage;
using system_analyzer::parseApplicationTcpSockets;
using system_analyzer::SocketBytes;
using system_analyzer::TcpSocketRecord;

namespace
{

    // A realistic /proc/net/tcp header line.
    const std::string kHeader =
        "  sl  local_address rem_address   st tx_queue rx_queue "
        "tr tm->when retrnsmt   uid  timeout inode\n";

} // namespace

int main()
{
    // (1) parseApplicationTcpSockets: extracts inode and state.
    {
        // Fixture uses the real /proc/net/tcp merged format:
        //   sl local rem st txq:rxq tr:when retrnsmt uid timeout inode ref...
        const std::string content =
            kHeader +
            "  0: 0100007F:1F90 0100007F:1F92 01 "
            "00000000:00000000 00:00000000 00000000 1000 0 101100 1\n"
            "  1: 0100007F:1F91 0100007F:1F93 01 "
            "00000000:00000000 00:00000000 00000000 1000 0 202200 1\n";

        const auto sockets = parseApplicationTcpSockets(content);

        assert(sockets.size() == 2);
        assert(sockets[0].inode == 101100);
        assert(sockets[0].state == 1);        // TCP_ESTABLISHED
        assert(sockets[0].localPort == 8080); // 0x1F90
        assert(sockets[1].inode == 202200);
    }

    // (2) Malformed / empty lines are skipped, never fatal.
    {
        const std::string content = kHeader + "\nnot_a_real_line\n  0:\n";
        assert(parseApplicationTcpSockets(content).empty());
    }

    // (3) Aggregation: multiple sockets for one app sum into one entry;
    // separate apps stay separate.
    {
        const std::vector<TcpSocketRecord> sockets = {
            {0x0100007F, 8080, 1, 101100},
            {0x0100007F, 8081, 1, 101101},
            {0x0100007F, 443, 1, 202200},
        };

        std::unordered_map<std::uint64_t, int> inodePid = {
            {101100, 4217}, {101101, 4217}, {202200, 5081}};

        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes = {
            {101100, {1000, 500}},
            {101101, {3000, 1500}},
            {202200, {200000, 40000}},
        };

        std::map<int, std::string> exeOf = {
            {4217, "/usr/lib/firefox/firefox"},
            {5081, "/usr/bin/google-chrome"}};

        std::map<int, std::string> commOf = {
            {4217, "firefox"}, {5081, "chrome"}};

        const auto usage = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        // Sorted by appId: chrome (alpha) before firefox.
        assert(usage.size() == 2);

        // chrome
        assert(usage[0].appId == "/usr/bin/google-chrome");
        assert(usage[0].executablePath == "/usr/bin/google-chrome");
        assert(usage[0].processName == "chrome");
        assert(usage[0].bytesReceived == 200000);
        assert(usage[0].bytesSent == 40000);

        // firefox: two sockets summed
        assert(usage[1].appId == "/usr/lib/firefox/firefox");
        assert(usage[1].processName == "firefox");
        assert(usage[1].bytesReceived == 1000 + 3000);
        assert(usage[1].bytesSent == 500 + 1500);
    }

    // (4) Inaccessible/vanished process: exe "" is skipped.
    {
        const std::vector<TcpSocketRecord> sockets = {
            {0x0100007F, 8080, 1, 101100}};

        std::unordered_map<std::uint64_t, int> inodePid = {{101100, 9999}};
        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes = {
            {101100, {100, 200}}};

        // pid 9999 has no exe entry (vanished/inaccessible).
        std::map<int, std::string> exeOf;
        std::map<int, std::string> commOf;

        const auto usage = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        assert(usage.empty()); // no fabrications
    }

    // (5) Unresolved inode (no pid owner) is skipped.
    {
        const std::vector<TcpSocketRecord> sockets = {
            {0x0100007F, 8080, 1, 303300}};

        std::unordered_map<std::uint64_t, int> inodePid; // empty
        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes;
        std::map<int, std::string> exeOf;
        std::map<int, std::string> commOf;

        const auto usage = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        assert(usage.empty());
    }

    // (6) Listener (state 0x0A) sockets carry no bytes -> not attributed.
    {
        const std::vector<TcpSocketRecord> sockets = {
            {0x0100007F, 8080, 10, 101100}}; // state 0x0A = LISTEN

        std::unordered_map<std::uint64_t, int> inodePid = {{101100, 4217}};
        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes = {
            {101100, {999, 999}}};

        std::map<int, std::string> exeOf = {{4217, "/usr/bin/sshd"}};
        std::map<int, std::string> commOf = {{4217, "sshd"}};

        const auto usage = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        assert(usage.empty());
    }

    // (7) Dedup: the same inode appearing twice (shared descriptor) counts
    // once per pid.
    {
        const std::vector<TcpSocketRecord> sockets = {
            {0x0100007F, 8080, 1, 101100},
            {0x0100007F, 8080, 1, 101100}, // duplicate
        };

        std::unordered_map<std::uint64_t, int> inodePid = {{101100, 4217}};
        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes = {
            {101100, {1000, 2000}}};

        std::map<int, std::string> exeOf = {{4217, "/usr/bin/tool"}};
        std::map<int, std::string> commOf = {{4217, "tool"}};

        const auto usage = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        assert(usage.size() == 1);
        assert(usage[0].bytesReceived == 1000); // not doubled
        assert(usage[0].bytesSent == 2000);
    }

    std::cout << "linux application network provider test passed\n";
    return 0;
}
