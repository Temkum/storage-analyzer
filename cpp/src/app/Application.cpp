#include <chrono>
#include "system_analyzer/app/Application.hpp"

#include "system_analyzer/core/DirectorySizeAggregator.hpp"
#include "system_analyzer/platform/linux/LinuxDiskUsageProvider.hpp"
#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

namespace system_analyzer::app
{

    domain::ScanResult Application::scan(
        const std::filesystem::path &root)
    {
        const auto start = std::chrono::steady_clock::now();

        using core::DirectorySizeAggregator;
        using platform::linux::LinuxDiskUsageProvider;
        using platform::linux::LinuxFileScanner;

        domain::ScanResult result;
        result.rootPath = root;

        LinuxFileScanner scanner;
        DirectorySizeAggregator aggregator;
        LinuxDiskUsageProvider diskUsageProvider;

        result.diskUsage = diskUsageProvider.getUsage(root);

        scanner.scan(
            root,
            [&result, &aggregator](
                const domain::FileEntry &entry)
            {
                result.entries.push_back(entry);
                aggregator.add(entry);

                switch (entry.type)
                {
                case domain::FileType::File:
                    ++result.fileCount;
                    break;

                case domain::FileType::Directory:
                    ++result.directoryCount;
                    break;

                default:
                    break;
                }
            },
            [&result](
                const std::filesystem::path &path,
                const std::error_code &error)
            {
                result.errors.push_back({path.string(),
                                         error.message()});
            });

        for (const auto &entry : result.entries)
        {
            if (entry.type != domain::FileType::Directory)
            {
                continue;
            }

            result.directories.push_back({entry.path,
                                          aggregator.sizeOf(entry.path)});
        }

        result.totalSize = aggregator.sizeOf(root);

        result.directories.push_back({root,
                                      result.totalSize});

        const auto end = std::chrono::steady_clock::now();

        result.durationMs =
            static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::milliseconds>(
                    end - start)
                    .count());

        return result;
    }

} // namespace system_analyzer::app