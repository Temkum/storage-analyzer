#include <iostream>
#include <stdexcept>
#include <string>

#include <nlohmann/json.hpp>

#include <sys/wait.h>
#include <unistd.h>

/// Long-lived sidecar integration test.
///
/// Spawns the real `system-analyzer --network` process, drives it over
/// stdin/stdout and proves: repeated snapshots served by ONE process,
/// explicit shutdown acknowledgement, and a clean exit code. A provider
/// unit test cannot prove that the sidecar is long-lived; this one can.

namespace
{

    [[noreturn]] void fail(
        const char *message)
    {
        throw std::runtime_error(message);
    }

    void require(
        bool condition,
        const char *message)
    {
        if (!condition)
        {
            fail(message);
        }
    }

    void sendRequest(
        int fd,
        const char *request)
    {
        const std::string line = std::string(request) + "\n";

        const ssize_t written = write(fd, line.data(), line.size());

        require(written == static_cast<ssize_t>(line.size()),
                "failed to write request to sidecar");
    }

} // namespace

int main(
    int argc,
    char *argv[])
{
    if (argc != 2)
    {
        std::cerr << "usage: network-sidecar-integration-test <sidecar-path>\n";
        return 2;
    }

    const char *sidecarPath = argv[1];

    try
    {
        int stdinPipe[2];
        int stdoutPipe[2];

        require(pipe(stdinPipe) == 0, "pipe() failed");
        require(pipe(stdoutPipe) == 0, "pipe() failed");

        const pid_t pid = fork();

        require(pid >= 0, "fork() failed");

        if (pid == 0)
        {
            // Child: wire the pipes to stdin/stdout and exec the sidecar.
            dup2(stdinPipe[0], STDIN_FILENO);
            dup2(stdoutPipe[1], STDOUT_FILENO);

            close(stdinPipe[0]);
            close(stdinPipe[1]);
            close(stdoutPipe[0]);
            close(stdoutPipe[1]);

            execl(sidecarPath, sidecarPath, "--network",
                  static_cast<char *>(nullptr));

            _exit(127); // exec failed
        }

        // Parent: keep the write end open until shutdown, read line by line.
        close(stdinPipe[0]);
        close(stdoutPipe[1]);

        FILE *childStdout = fdopen(stdoutPipe[0], "r");

        require(childStdout != nullptr, "fdopen() failed");

        char buffer[65536];

        auto readResponse = [&]() -> std::string
        {
            const char *received =
                fgets(buffer, sizeof buffer, childStdout);

            require(received != nullptr,
                    "sidecar closed stdout before responding");

            return std::string(buffer);
        };

        // Two snapshot requests served by the SAME process prove it is
        // long-lived rather than one-shot like the disk scan sidecar.
        for (int i = 0; i < 2; ++i)
        {
            sendRequest(stdinPipe[1], R"({"command":"network_snapshot"})");

            const nlohmann::json response =
                nlohmann::json::parse(readResponse());

            require(response["type"] == "network_snapshot",
                    "expected a network_snapshot response");
            require(response.contains("timestamp"),
                    "snapshot response missing timestamp");
            require(response["interfaces"].is_array(),
                    "snapshot response missing interfaces");
        }

        sendRequest(stdinPipe[1], R"({"command":"shutdown"})");

        const nlohmann::json ack = nlohmann::json::parse(readResponse());

        require(ack["type"] == "shutdown_ack",
                "expected a shutdown_ack response");

        fclose(childStdout);
        close(stdinPipe[1]);

        int status = 0;

        require(waitpid(pid, &status, 0) == pid, "waitpid() failed");
        require(WIFEXITED(status), "sidecar did not exit normally");
        require(WEXITSTATUS(status) == 0,
                "sidecar exited with a failure status");

        std::cout << "network sidecar integration test passed\n";

        return 0;
    }
    catch (const std::exception &error)
    {
        std::cerr << "network sidecar integration failure: "
                  << error.what()
                  << '\n';
        return 1;
    }
}