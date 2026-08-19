#include <cassert>
#include <filesystem>

#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

int main()
{
    using system_analyzer::platform::linux::LinuxFileScanner;

    LinuxFileScanner scanner;

    bool errorReported = false;

    scanner.scan(
        "/tmp/system-analyzer-path-that-does-not-exist",
        [](const auto &) {},
        [&](const std::filesystem::path &path,
            const std::error_code &error)
        {
            errorReported = true;

            assert(path ==
                   "/tmp/system-analyzer-path-that-does-not-exist");

            assert(error == std::errc::no_such_file_or_directory);
        });

    assert(errorReported);

    return 0;
}
