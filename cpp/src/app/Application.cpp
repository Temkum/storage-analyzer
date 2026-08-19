#include "system_analyzer/app/Application.hpp"

#include "system_analyzer/core/DirectorySizeAggregator.hpp"
#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

namespace system_analyzer::app
{

    domain::ScanResult Application::scan(
        const std::filesystem::path &root)
    {
        using core::DirectorySizeAggregator;
        using platform::linux::LinuxFileScanner;

        domain::ScanResult result;
        result.rootPath = root;

        LinuxFileScanner scanner;
        DirectorySizeAggregator aggregator;

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

        return result;
    }

} // namespace system_analyzer::app