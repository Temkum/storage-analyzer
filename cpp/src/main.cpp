#include <iostream>

#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

namespace
{

    const char *fileTypeToString(
        system_analyzer::domain::FileType type)
    {
        using system_analyzer::domain::FileType;

        switch (type)
        {
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

int main()
{
    using system_analyzer::platform::linux::LinuxFileScanner;

    LinuxFileScanner scanner;

    scanner.scan(
        "/tmp/system-analyzer-test",
        [](const system_analyzer::domain::FileEntry &entry)
        {
            std::cout
                << fileTypeToString(entry.type)
                << " | "
                << entry.size
                << " bytes | "
                << entry.path
                << '\n';
        });

    return 0;
}