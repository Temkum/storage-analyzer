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

        LinuxFileScanner scanner;
        DirectorySizeAggregator aggregator;

        scanner.scan(
            root,
            [&result, &aggregator](
                const domain::FileEntry &entry)
            {
                result.entries.push_back(entry);
                aggregator.add(entry);
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

        result.directories.push_back({root,
                                      aggregator.sizeOf(root)});

        return result;
    }

} // namespace system_analyzer::app