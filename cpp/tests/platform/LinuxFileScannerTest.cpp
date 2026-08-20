#include <cassert>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <string>

#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

int main()
{
    using system_analyzer::core::ScanContext;
    using system_analyzer::platform::linux::LinuxFileScanner;

    // missing root reports an error
    {
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
    }

    // real directory reports progress for every entry
    {
        const std::filesystem::path tempRoot =
            std::filesystem::temp_directory_path() /
            "linux-file-scanner-progress";

        std::filesystem::remove_all(tempRoot);
        std::filesystem::create_directories(tempRoot / "sub");
        std::ofstream(tempRoot / "a.txt") << "a";
        std::ofstream(tempRoot / "b.txt") << "b";
        std::ofstream(tempRoot / "sub" / "c.txt") << "c";

        LinuxFileScanner scanner;

        std::uintmax_t entryCount = 0;
        std::uintmax_t lastProgress = 0;

        scanner.scan(
            tempRoot,
            [&](const auto &)
            {
                ++entryCount;
            },
            [](const std::filesystem::path &,
               const std::error_code &)
            {
                assert(false && "no error was expected");
            },
            ScanContext{
                [&](std::uintmax_t scanned)
                {
                    lastProgress = scanned;
                },
                []()
                { return false; }});

        assert(entryCount > 0);
        assert(lastProgress == entryCount);

        std::filesystem::remove_all(tempRoot);
    }

    // cancellation stops the scan early
    {
        const std::filesystem::path tempRoot =
            std::filesystem::temp_directory_path() /
            "linux-scanner-cancellation";

        std::filesystem::remove_all(tempRoot);
        std::filesystem::create_directories(tempRoot);

        for (int i = 0; i < 50; ++i)
        {
            std::ofstream(
                tempRoot / ("file" + std::to_string(i) + ".txt"));
        }

        LinuxFileScanner scanner;

        std::uintmax_t cancelledAfter = 7;
        std::uintmax_t seen = 0;

        scanner.scan(
            tempRoot,
            [&](const auto &)
            {
                ++seen;
            },
            [](const std::filesystem::path &,
               const std::error_code &) {},
            ScanContext{
                [](std::uintmax_t) {},
                [&]()
                { return seen >= cancelledAfter; }});

        assert(seen == cancelledAfter);

        std::filesystem::remove_all(tempRoot);
    }

    return 0;
}
