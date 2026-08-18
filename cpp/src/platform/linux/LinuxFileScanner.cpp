#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

#include <filesystem>
#include <system_error>

#include "system_analyzer/platform/linux/LinuxFileEntryMapper.hpp"

namespace system_analyzer::platform::linux
{

    void LinuxFileScanner::scan(
        const std::filesystem::path &root,
        const EntryCallback &callback)
    {
        std::error_code error;

        std::filesystem::recursive_directory_iterator iterator(
            root,
            std::filesystem::directory_options::skip_permission_denied,
            error);

        const std::filesystem::recursive_directory_iterator end;

        while (iterator != end)
        {
            if (error)
            {
                error.clear();
                iterator.increment(error);
                continue;
            }

            const auto &entry = *iterator;

            const auto fileEntry = LinuxFileEntryMapper::map(entry);

            callback(fileEntry);

            iterator.increment(error);
        }
    }

} // namespace system_analyzer::platform::linux