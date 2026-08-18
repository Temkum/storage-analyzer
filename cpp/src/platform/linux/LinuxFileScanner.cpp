#include "system_analyzer/platform/linux/LinuxFileScanner.hpp"

#include <filesystem>
#include <system_error>

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

            domain::FileEntry fileEntry{
                .path = entry.path(),
                .type = entry.is_directory()
                            ? domain::FileType::Directory
                        : entry.is_symlink()
                            ? domain::FileType::Symlink
                        : entry.is_regular_file()
                            ? domain::FileType::File
                            : domain::FileType::Other,
                .size = entry.is_regular_file(error)
                            ? entry.file_size(error)
                            : 0};

            callback(fileEntry);

            iterator.increment(error);
        }
    }

} // namespace system_analyzer::platform::linux
