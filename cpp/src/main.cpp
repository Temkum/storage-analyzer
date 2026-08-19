#include <filesystem>
#include <iostream>

#include "system_analyzer/app/Application.hpp"
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

    const auto result = application.scan(root);

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