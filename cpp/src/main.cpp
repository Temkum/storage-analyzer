#include <cstdint>
#include <filesystem>
#include <iostream>
#include <string>

#include "system_analyzer/app/Application.hpp"
#include "system_analyzer/core/ScanContext.hpp"
#include "system_analyzer/serialization/ScanResultSerializer.hpp"

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

int main(int argc, char *argv[])
{
    // Long-lived network sidecar mode: stays alive, reads one NDJSON command
    // per line from stdin and answers on stdout (see Application::
    // runNetworkMode). Kept alongside the existing disk-scan entry path.
    if (argc == 2 && std::string(argv[1]) == "--network")
    {
        system_analyzer::app::Application application;

        return application.runNetworkMode();
    }

    if (argc != 2)
    {
        std::cerr << "Usage: system-analyzer <directory>\n";
        return 1;
    }

    const std::filesystem::path root = argv[1];

    if (!std::filesystem::exists(root))
    {
        std::cerr << "Error: path does not exist: " << root << '\n';
        return 1;
    }

    if (!std::filesystem::is_directory(root))
    {
        std::cerr << "Error: path is not a directory: " << root << '\n';
        return 1;
    }

    system_analyzer::app::Application application;

    system_analyzer::core::ScanContext context;

    context.onProgress =
        [](std::uintmax_t scannedEntries)
        {
            std::cerr
                << "PROGRESS:"
                << scannedEntries
                << '\n';
            std::cerr.flush();
        };

    const auto result = application.scan(root, context);

    const auto json =
        system_analyzer::serialization::ScanResultSerializer::toJson(result);

    std::cout << json << '\n';

    return 0;

    /* for (const auto &entry : result.entries)
    {
        std::cout
            << fileTypeToString(entry.type)
            << " | "
            << entry.size
            << " bytes | "
            << entry.path
            << '\n';
    }

    std::cout << "\nDirectory sizes:\n";

    for (const auto &directory : result.directories)
    {
        std::cout
            << directory.path
            << " = "
            << directory.size
            << " bytes\n";
    }

    return 0; */
}