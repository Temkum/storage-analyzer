#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

#include <cstdint>
#include <filesystem>
#include <system_error>

#include "system_analyzer/platform/linux/LinuxFileEntryMapper.hpp"

namespace system_analyzer::platform::linux
{

    void LinuxFileScanner::scan(
        const std::filesystem::path &root,
        const EntryCallback &entryCallback,
        const ErrorCallback &errorCallback,
        const core::ScanContext &context)
    {
        std::error_code error;

        std::filesystem::recursive_directory_iterator iterator(
            root,
            std::filesystem::directory_options::skip_permission_denied,
            error);

        const std::filesystem::recursive_directory_iterator end;

        if (error)
        {
            errorCallback(root, error);
            return;
        }

        std::uintmax_t scannedEntries = 0;

        while (iterator != end)
        {
            if (context.isCancelled && context.isCancelled())
            {
                break;
            }

            if (error)
            {
                errorCallback(iterator->path(), error);

                error.clear();
                iterator.increment(error);
                continue;
            }

            const auto &entry = *iterator;

            const auto fileEntry =
                LinuxFileEntryMapper::map(entry);

            entryCallback(fileEntry);

            ++scannedEntries;

            if (context.onProgress)
            {
                context.onProgress(scannedEntries);
            }

            iterator.increment(error);
        }

        if (error)
        {
            errorCallback(root, error);
        }
    }

} // namespace system_analyzer::platform::linux