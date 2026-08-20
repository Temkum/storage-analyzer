#pragma once

#include <cstdint>
#include <filesystem>
#include <functional>

#include "system_analyzer/domain/FileEntry.hpp"

namespace system_analyzer::core
{

    struct ScanContext
    {
        std::function<void(std::uintmax_t)> onProgress = {};
        std::function<bool()> isCancelled = {};
    };

    class IFileScanner
    {
    public:
        using EntryCallback = std::function<void(const domain::FileEntry &)>;
        using ErrorCallback = std::function<void(
            const std::filesystem::path &,
            const std::error_code &)>;

        virtual ~IFileScanner() = default;

        virtual void scan(
            const std::filesystem::path &root,
            const EntryCallback &callback,
            const ErrorCallback &errorCallback,
            const ScanContext &context = {}) = 0;
    };

} // namespace system_analyzer::core