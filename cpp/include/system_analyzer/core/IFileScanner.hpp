#pragma once

#include <filesystem>
#include <functional>

#include "system_analyzer/domain/FileEntry.hpp"

namespace system_analyzer::core
{

    class IFileScanner
    {
    public:
        using EntryCallback = std::function<void(const domain::FileEntry &)>;

        virtual ~IFileScanner() = default;

        virtual void scan(
            const std::filesystem::path &root,
            const EntryCallback &callback) = 0;
    };

} // namespace system_analyzer::core