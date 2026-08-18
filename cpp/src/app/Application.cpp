#include "system_analyzer/app/Application.hpp"

#include <iostream>

#include "system_analyzer/core/DirectorySizeAggregator.hpp"
#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

namespace {

const char* fileTypeToString(
    system_analyzer::domain::FileType type
) {
    using system_analyzer::domain::FileType;

    switch (type) {
        case FileType::File:
            return "FILE";

        case FileType::Directory:
            return "DIR";

        case FileType::Symlink:
            return "LINK";

        case FileType::Other:
            return "OTHER";
    }

    return "UNKNOWN";
}

} // namespace

namespace system_analyzer::app {

int Application::run(const std::filesystem::path& root) {
    using core::DirectorySizeAggregator;
    using platform::linux::LinuxFileScanner;

    LinuxFileScanner scanner;
    DirectorySizeAggregator aggregator;

    scanner.scan(
        root,
        [&aggregator](const domain::FileEntry& entry) {
            aggregator.add(entry);

            std::cout
                << fileTypeToString(entry.type)
                << " | "
                << entry.size
                << " bytes | "
                << entry.path
                << '\n';
        }
    );

    std::cout << "\nDirectory sizes:\n";

    std::cout
        << root
        << " = "
        << aggregator.sizeOf(root)
        << " bytes\n";

    std::cout
        << root / "subdir"
        << " = "
        << aggregator.sizeOf(root / "subdir")
        << " bytes\n";

    return 0;
}

} // namespace system_analyzer::app
