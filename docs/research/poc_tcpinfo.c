// Phase 6 mechanism validation (unprivileged, read-only).
//
// Establishes on the running kernel:
//   F1: per-socket cumulative TCP byte counters exist (TCP_INFO) and are
//       readable without privileges for sockets we own.
//   F2: the /proc identity chain works: /proc/net/tcp socket inode
//       -> /proc/<pid>/fd/socket:[inode] -> /proc/<pid>/exe.
//   F3: unresolvable inodes are skipped gracefully (the Step 6.1 rule).
//   F4: /proc/net/udp exposes NO cumulative byte counters (polling gap).
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dirent.h>
#include <errno.h>
#include <linux/tcp.h> /* kernel UAPI: glibc's netinet/tcp.h struct tcp_info is outdated */
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdint.h>
#include <sys/wait.h>

#define PAYLOAD_TX 1000000u /* client -> server */
#define PAYLOAD_RX 500000u  /* server -> client */
#define FAKE_INODE 424242424242UL

static int write_all(int fd, const unsigned char *buf, size_t len)
{
    size_t done = 0;
    while (done < len)
    {
        size_t chunk = len - done;
        ssize_t n;
        if (chunk > 65536)
            chunk = 65536;
        n = send(fd, buf, chunk, 0); /* buf is zero-filled scratch, always the same region */
        if (n <= 0)
            return -1;
        done += (size_t)n;
    }
    return 0;
}

static int read_all(int fd, size_t len)
{
    unsigned char buf[65536];
    size_t done = 0;
    while (done < len)
    {
        ssize_t n = recv(fd, buf, sizeof buf, 0);
        if (n <= 0)
            return -1;
        done += (size_t)n;
    }
    return 0;
}

static void print_tcp_info(const char *label, int fd)
{
    struct tcp_info ti;
    socklen_t sl = sizeof ti;
    memset(&ti, 0, sizeof ti);
    if (getsockopt(fd, IPPROTO_TCP, TCP_INFO, &ti, &sl) != 0)
    {
        printf("%s: getsockopt(TCP_INFO) failed: %s\n", label, strerror(errno));
        exit(2);
    }
    printf("%s: bytes_sent=%llu bytes_acked=%llu bytes_received=%llu "
           "data_segs_out=%u data_segs_in=%u\n",
           label,
           (unsigned long long)ti.tcpi_bytes_sent,
           (unsigned long long)ti.tcpi_bytes_acked,
           (unsigned long long)ti.tcpi_bytes_received,
           ti.tcpi_data_segs_out, ti.tcpi_data_segs_in);
}

/* Find the kernel socket inode for a given local port in /proc/net/tcp. */
static long inode_for_local_port(int port, unsigned expect_state)
{
    FILE *f = fopen("/proc/net/tcp", "r");
    char line[512];
    long found = -1;

    if (!f)
        return -1;
    if (!fgets(line, sizeof line, f)) /* header */
    {
        fclose(f);
        return -1;
    }
    while (fgets(line, sizeof line, f))
    {
        int sl;
        char la[64], ra[64], txrx[64], trwhen[64];
        unsigned st, retrnsmt, uid, timeout;
        unsigned long inode;

        if (sscanf(line, "%d: %63s %63s %X %63s %63s %X %u %u %lu",
                   &sl, la, ra, &st, txrx, trwhen, &retrnsmt,
                   &uid, &timeout, &inode) != 10)
        {
#ifdef POC_DEBUG
            fprintf(stderr, "DBG parse-fail: %s", line);
#endif
            continue;
        }

        const char *colon = strchr(la, ':');
        if (!colon)
            continue;

#ifdef POC_DEBUG
        fprintf(stderr, "DBG la=%s st=%u port=%d target=%d\n",
                la, st, (int)strtol(colon + 1, NULL, 16), port);
#endif
        if ((int)strtol(colon + 1, NULL, 16) == port && st == expect_state)
        {
            found = (long)inode;
            break;
        }
    }
    fclose(f);
    return found;
}

/* Map socket inode -> owning pid via /proc/<pid>/fd. Returns pid or -1.
 * Tolerates processes that disappear (ENOENT) and unreadable fd dirs
 * (EACCES) by skipping them — the Step 6.1 permission rule. */
static pid_t pid_for_inode(unsigned long inode)
{
    DIR *proc = opendir("/proc");
    struct dirent *de;
    pid_t found = -1;

    if (!proc)
        return -1;

    while ((de = readdir(proc)) != NULL)
    {
        char path[320];
        pid_t pid;
        DIR *fds;
        struct dirent *fde;

        if (de->d_name[0] < '0' || de->d_name[0] > '9')
            continue;
        pid = (pid_t)atoi(de->d_name);
        snprintf(path, sizeof path, "/proc/%d/fd", pid);

        fds = opendir(path);
        if (!fds) /* permission denied or process gone: skip */
            continue;

        while ((fde = readdir(fds)) != NULL)
        {
            char target[256];
            ssize_t n;
            snprintf(path, sizeof path, "/proc/%d/fd/%s", pid, fde->d_name);
            n = readlink(path, target, sizeof target - 1);
            if (n <= 0)
                continue;
            target[n] = '\0';
            if (strncmp(target, "socket:[", 8) == 0)
            {
                unsigned long fd_inode = strtoul(target + 8, NULL, 10);
                if (fd_inode == inode)
                {
                    found = pid;
                    break;
                }
            }
        }
        closedir(fds);
        if (found > 0)
            break;
    }
    closedir(proc);
    return found;
}

static void print_exe(pid_t pid)
{
    char path[64], target[512];
    ssize_t n;
    snprintf(path, sizeof path, "/proc/%d/exe", pid);
    n = readlink(path, target, sizeof target - 1);
    if (n < 0)
    {
        printf("    exe: UNREADABLE (%s)\n", strerror(errno));
        return;
    }
    target[n] = '\0';
    printf("    exe: %s\n", target);
}

/* server side runs in the child; `release_fd` keeps it alive (and the
 * connection ESTAB) until the parent finishes its /proc lookups. */
static void run_server(int port, int release_fd)
{
    int srv = socket(AF_INET, SOCK_STREAM, 0);
    struct sockaddr_in addr;
    int conn;
    char release;

    memset(&addr, 0, sizeof addr);
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = htons((uint16_t)port);

    if (setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &(int){1}, sizeof(int)) != 0 ||
        bind(srv, (struct sockaddr *)&addr, sizeof addr) != 0 ||
        listen(srv, 1) != 0)
    {
        perror("server setup");
        exit(2);
    }

    {
        socklen_t alen = sizeof addr;
        conn = accept(srv, (struct sockaddr *)&addr, &alen);
    }
    if (conn < 0)
    {
        perror("accept");
        exit(2);
    }

    if (read_all(conn, PAYLOAD_TX) != 0)
    {
        perror("server read");
        exit(2);
    }
    {
        unsigned char *sbuf = calloc(1, 65536);
        if (write_all(conn, sbuf, PAYLOAD_RX) != 0)
        {
            perror("server write");
            exit(2);
        }
        free(sbuf);
    }
    print_tcp_info("SERVER", conn);

    /* Hold the connection open until the parent releases us. Closing here
     * would send FIN and move the parent's socket into CLOSE_WAIT. */
    (void)!read(release_fd, &release, 1);

    close(conn);
    close(srv);
    exit(0);
}

int main(void)
{
    int port_pipe[2]; /* child -> parent: chosen listen port */
    int rel_pipe[2];  /* parent -> child: "release and exit" */
    struct sockaddr_in addr;
    socklen_t len = sizeof addr;
    pid_t server_pid, self = getpid();
    unsigned char *buf;
    int port = 0;

    if (pipe(port_pipe) != 0 || pipe(rel_pipe) != 0)
    {
        perror("pipe");
        return 2;
    }

    server_pid = fork();
    if (server_pid == 0)
    {
        close(port_pipe[0]);
        close(rel_pipe[1]);
        /* Parent seeds a starting port so repeated runs don't contend for
         * the same fixed range concurrently. */
        srand((unsigned)(getpid() ^ (uintptr_t)&port_pipe));
        int port_hint = 21100 + (rand() % 30000);
        for (int attempt = 0; attempt < 64; attempt++)
        {
            int probe = socket(AF_INET, SOCK_STREAM, 0);
            struct sockaddr_in a;
            memset(&a, 0, sizeof a);
            a.sin_family = AF_INET;
            a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
            a.sin_port = htons((uint16_t)(port_hint + attempt));
            if (bind(probe, (struct sockaddr *)&a, sizeof a) == 0)
            {
                int port_chosen = port_hint + attempt;
                close(probe);
                (void)!write(port_pipe[1], &port_chosen, sizeof port_chosen);
                close(port_pipe[1]);
                run_server(port_chosen, rel_pipe[0]); /* exits on release */
            }
            close(probe);
        }
        exit(2);
    }
    close(port_pipe[1]);
    close(rel_pipe[0]);
    if (read(port_pipe[0], &port, sizeof port) != sizeof port)
    {
        perror("read port");
        return 2;
    }
    close(port_pipe[0]);

    {
        int cli = socket(AF_INET, SOCK_STREAM, 0);
        memset(&addr, 0, sizeof addr);
        addr.sin_family = AF_INET;
        addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
        addr.sin_port = htons((uint16_t)port);

        if (connect(cli, (struct sockaddr *)&addr, sizeof addr) != 0)
        {
            perror("connect");
            return 2;
        }

        buf = calloc(1, 65536);
        if (write_all(cli, buf, PAYLOAD_TX) != 0 || read_all(cli, PAYLOAD_RX) != 0)
        {
            perror("client transfer");
            return 2;
        }
        free(buf);

        printf("== F1: per-socket cumulative TCP counters (TCP_INFO) ==\n");
        print_tcp_info("CLIENT", cli);

        printf("== F2: identity chain /proc/net/tcp inode -> pid -> exe ==\n");
        {
            int local_port = 0;
            getsockname(cli, (struct sockaddr *)&addr, &len);
            local_port = ntohs(addr.sin_port);
            {
                long inode = inode_for_local_port(local_port, 0x01 /* ESTAB */);
                printf("  client local port %d -> socket inode %ld\n", local_port, inode);
                if (inode < 0)
                {
                    printf("  FAIL: connection not found in /proc/net/tcp\n");
                    return 1;
                }
                {
                    pid_t owner = pid_for_inode((unsigned long)inode);
                    printf("  inode -> pid: %d (expected %d)\n", owner, self);
                    if (owner > 0)
                        print_exe(owner);
                }
            }
        }

        printf("== F3: unresolvable inode is skipped, not fatal ==\n");
        {
            pid_t nobody = pid_for_inode(FAKE_INODE);
            printf("  fake inode %lu -> pid %d (graceful skip: %s)\n",
                   FAKE_INODE, nobody, nobody < 0 ? "OK" : "FAIL");
        }

        printf("== F4: UDP has no cumulative byte counters ==\n");
        {
            int u = socket(AF_INET, SOCK_DGRAM, 0);
            struct sockaddr_in ua;
            memset(&ua, 0, sizeof ua);
            ua.sin_family = AF_INET;
            ua.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
            ua.sin_port = 0;
            if (bind(u, (struct sockaddr *)&ua, sizeof ua) == 0 &&
                getsockname(u, (struct sockaddr *)&ua, &len) == 0)
            {
                int uport = ntohs(ua.sin_port);
                FILE *f = fopen("/proc/net/udp", "r");
                char line[512];
                int seen = 0;
                if (f)
                {
                    if (fgets(line, sizeof line, f)) /* header */
                        printf("  header: %s", line);
                    while (fgets(line, sizeof line, f))
                    {
                        int sl;
                        char la[64], ra[64], txrx[64], trwhen[64];
                        unsigned st, retrnsmt, uid, timeout, drops;
                        unsigned long inode;

                        if (sscanf(line, "%d: %63s %63s %X %63s %63s %X %u %u %lu %*u %*s %u",
                                   &sl, la, ra, &st, txrx, trwhen, &retrnsmt,
                                   &uid, &timeout, &inode, &drops) < 9)
                            continue;
                        {
                            const char *colon = strchr(la, ':');
                            if (colon && (int)strtol(colon + 1, NULL, 16) == uport)
                            {
                                printf("  udp row: %s", line);
                                seen = 1;
                                break;
                            }
                        }
                    }
                    fclose(f);
                }
                printf("  -> row shows queues/drops only, no cumulative bytes; "
                       "socket found: %s\n",
                       seen ? "yes" : "no");
            }
            close(u);
        }

        close(cli);
        /* Now release the server child so it can drain and exit. */
        {
            unsigned char go = 1;
            (void)!write(rel_pipe[1], &go, 1);
        }
    }
    close(rel_pipe[1]);
    {
        int status;
        waitpid(server_pid, &status, 0);
    }
    return 0;
}
