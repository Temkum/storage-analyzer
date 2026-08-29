#include "system_analyzer/platform/linux/LinuxApplicationNetworkProvider.hpp"

#include <dirent.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

// CRITICAL: glibc's <netinet/tcp.h> does not define tcpi_bytes_sent /
// tcpi_bytes_received. Use the kernel UAPI header instead.
// Do NOT "clean up" this include — the provider silently reports zero
// traffic without it.
#include <linux/tcp.h>
#include <linux/rtnetlink.h>

// SOCK_DIAG / netlink for querying TCP_INFO on a socket by inode.
#include <linux/netlink.h>
#include <linux/sock_diag.h>
#include <linux/inet_diag.h>

#include "system_analyzer/platform/linux/LinuxApplicationNetworkParser.hpp"

#include <cerrno>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <ctime>
#include <fstream>
#include <sstream>
#include <string>
#include <unordered_map>

namespace system_analyzer
{

    namespace
    {
        // Kernel TCP state constants. Defined locally to avoid pulling in
        // <netinet/tcp.h>, which conflicts with <linux/tcp.h>.
        constexpr int kTcpEstablished = 1;

        // Enumerate all numeric PID directories under /proc.
        std::vector<pid_t> enumeratePids()
        {
            std::vector<pid_t> pids;
            DIR *dir = opendir("/proc");
            if (!dir)
            {
                return pids;
            }

            dirent *entry;
            while ((entry = readdir(dir)) != nullptr)
            {
                if (entry->d_type != DT_DIR &&
                    entry->d_type != DT_UNKNOWN)
                {
                    continue;
                }

                const std::string name(entry->d_name);
                if (name.empty())
                {
                    continue;
                }

                bool allDigits = true;
                for (char c : name)
                {
                    if (!std::isdigit(static_cast<unsigned char>(c)))
                    {
                        allDigits = false;
                        break;
                    }
                }
                if (!allDigits)
                {
                    continue;
                }

                try
                {
                    pids.push_back(static_cast<pid_t>(std::stoi(name)));
                }
                catch (...)
                {
                    continue;
                }
            }

            closedir(dir);
            return pids;
        }

        // Resolve a PID to its canonical executable path via /proc/<pid>/exe.
        std::string resolveExecutable(pid_t pid)
        {
            char path[256];
            std::snprintf(
                path, sizeof(path), "/proc/%d/exe", static_cast<int>(pid));

            char buf[4096];
            const ssize_t len = readlink(path, buf, sizeof(buf) - 1);
            if (len <= 0 || len >= static_cast<ssize_t>(sizeof(buf) - 1))
            {
                return "";
            }

            buf[len] = '\0';
            return std::string(buf);
        }

        // Map a PID's socket fds to inode -> pid. Sets accessible=false on
        // EACCES/EPERM so the caller can skip that process.
        void collectSocketInodes(
            pid_t pid,
            std::unordered_map<std::uint64_t, int> &out,
            bool &accessible)
        {
            accessible = true;

            char dirPath[256];
            std::snprintf(
                dirPath,
                sizeof(dirPath),
                "/proc/%d/fd",
                static_cast<int>(pid));

            DIR *dir = opendir(dirPath);
            if (!dir)
            {
                if (errno == EACCES || errno == EPERM)
                {
                    accessible = false;
                }
                return;
            }

            dirent *entry;
            while ((entry = readdir(dir)) != nullptr)
            {
                if (entry->d_name[0] == '.')
                {
                    continue;
                }

                char linkPath[512];
                std::snprintf(
                    linkPath,
                    sizeof(linkPath),
                    "/proc/%d/fd/%s",
                    static_cast<int>(pid),
                    entry->d_name);

                char linkTarget[256];
                const ssize_t len = readlink(
                    linkPath, linkTarget, sizeof(linkTarget) - 1);
                if (len <= 0)
                {
                    continue;
                }
                linkTarget[len] = '\0';

                const std::string target(linkTarget);
                if (target.substr(0, 8) != "socket:[")
                {
                    continue;
                }

                const auto closeBracket = target.find(']');
                if (closeBracket == std::string::npos || closeBracket <= 8)
                {
                    continue;
                }

                const std::string inodeStr = target.substr(8, closeBracket - 8);

                try
                {
                    std::uint64_t inode = std::stoull(inodeStr);
                    // First process wins for shared descriptors.
                    if (out.find(inode) == out.end())
                    {
                        out[inode] = static_cast<int>(pid);
                    }
                }
                catch (...)
                {
                    continue;
                }
            }

            closedir(dir);
        }

        // Dump every TCP socket's cumulative byte counters (via SOCK_DIAG /
        // INET_DIAG_INFO, the same mechanism `ss -eit` uses to read TCP_INFO
        // for sockets owned by other processes). Returns inode -> bytes for
        // every socket the kernel reports. Works unprivileged for same-uid
        // sockets. The dump is a multi-chunk netlink response, so we loop
        // recv() until NLMSG_DONE; a single recv only yields the first chunk.
        std::unordered_map<std::uint64_t, SocketBytes> queryAllTcpBytes()
        {
            std::unordered_map<std::uint64_t, SocketBytes> result;

            struct
            {
                struct nlmsghdr nlh;
                struct inet_diag_req_v2 req;
            } request{};

            request.nlh.nlmsg_len = sizeof(request);
            // SOCK_DIAG_BY_FAMILY (NOT TCPDIAG_GETSOCK) is the correct dump
            // opcode: TCPDIAG_GETSOCK + NLM_F_DUMP is rejected with EINVAL.
            request.nlh.nlmsg_type = SOCK_DIAG_BY_FAMILY;
            request.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_DUMP;

            request.req.sdiag_family = AF_INET;
            request.req.sdiag_protocol = IPPROTO_TCP;
            request.req.idiag_states = (1 << kTcpEstablished);
            // request the TCP_INFO attribute (INET_DIAG_INFO).
            request.req.idiag_ext = (1 << (INET_DIAG_INFO - 1));

            static std::uint32_t seqCounter = 1;
            request.nlh.nlmsg_seq = seqCounter++;

            const int fd = socket(AF_NETLINK, SOCK_RAW, NETLINK_SOCK_DIAG);
            if (fd < 0)
            {
                return result;
            }

            sockaddr_nl peer{};
            peer.nl_family = AF_NETLINK;

            const ssize_t sent = sendto(fd, &request, sizeof(request), 0,
                                        reinterpret_cast<sockaddr *>(&peer),
                                        sizeof(peer));
            if (sent < 0)
            {
                close(fd);
                return result;
            }

            for (;;)
            {
                char buf[262144];
                const ssize_t recvd = recv(fd, buf, sizeof(buf), 0);
                if (recvd < 0)
                {
                    break;
                }

                struct nlmsghdr *nlh = reinterpret_cast<struct nlmsghdr *>(buf);
                int rem = static_cast<int>(recvd);
                bool done = false;

                for (; NLMSG_OK(nlh, rem); nlh = NLMSG_NEXT(nlh, rem))
                {
                    if (nlh->nlmsg_type == NLMSG_DONE)
                    {
                        done = true;
                        break;
                    }
                    if (nlh->nlmsg_type == NLMSG_ERROR)
                    {
                        continue;
                    }

                    struct inet_diag_msg *msg =
                        reinterpret_cast<struct inet_diag_msg *>(
                            NLMSG_DATA(nlh));

                    int attrLen = static_cast<int>(
                        nlh->nlmsg_len - NLMSG_LENGTH(sizeof(*msg)));
                    struct rtattr *attr =
                        reinterpret_cast<struct rtattr *>(msg + 1);

                    for (; RTA_OK(attr, attrLen);
                         attr = RTA_NEXT(attr, attrLen))
                    {
                        if (attr->rta_type == INET_DIAG_INFO)
                        {
                            struct tcp_info info{};
                            std::size_t payloadLen = RTA_PAYLOAD(attr);
                            std::memcpy(
                                &info, RTA_DATA(attr),
                                std::min(payloadLen, sizeof(info)));

                            SocketBytes bytes;
                            bytes.bytesSent = info.tcpi_bytes_sent;
                            bytes.bytesReceived = info.tcpi_bytes_received;
                            result[msg->idiag_inode] = bytes;
                            break;
                        }
                    }
                }

                if (done)
                {
                    break;
                }
            }

            close(fd);
            return result;
        }

    } // anonymous namespace

    ApplicationNetworkSnapshot
    LinuxApplicationNetworkProvider::getSnapshot()
    {
        ApplicationNetworkSnapshot snapshot;
        snapshot.timestamp =
            static_cast<std::uint64_t>(std::time(nullptr));

        std::ifstream tcpFile("/proc/net/tcp");
        if (!tcpFile)
        {
            // /proc unavailable (containers) — empty snapshot.
            return snapshot;
        }

        std::stringstream ss;
        ss << tcpFile.rdbuf();
        tcpFile.close();

        std::vector<TcpSocketRecord> sockets =
            parseApplicationTcpSockets(ss.str());

        // Build inode -> pid by enumerating all processes.
        std::unordered_map<std::uint64_t, int> inodePid;
        for (pid_t pid : enumeratePids())
        {
            bool accessible = true;
            collectSocketInodes(pid, inodePid, accessible);
            if (!accessible)
            {
                // Permission denied — skip this process per the contract.
                continue;
            }
        }

        // Resolve pids -> exe and comm (display name).
        std::map<int, std::string> exeOf;
        std::map<int, std::string> commOf;
        for (const auto &[inode, pid] : inodePid)
        {
            if (exeOf.find(pid) == exeOf.end())
            {
                exeOf[pid] = resolveExecutable(pid);
            }
            if (commOf.find(pid) == commOf.end())
            {
                char path[256];
                std::snprintf(path, sizeof(path), "/proc/%d/comm",
                              static_cast<int>(pid));
                std::ifstream comm(path);
                std::string name;
                if (std::getline(comm, name))
                {
                    commOf[pid] = name;
                }
            }
        }

        // Query byte counters for ALL established sockets in one SOCK_DIAG
        // dump. Sockets whose query failed or vanished are simply absent
        // from the map and get zero counters; never fail the whole snapshot.
        std::unordered_map<std::uint64_t, SocketBytes> inodeBytes =
            queryAllTcpBytes();

        snapshot.applications = aggregateApplicationUsage(
            sockets, inodePid, inodeBytes, exeOf, commOf);

        return snapshot;
    }

} // namespace system_analyzer
