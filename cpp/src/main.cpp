#include <filesystem>
#include <iostream>

#include "system_analyzer/app/Application.hpp"

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

    return application.run(root);
}