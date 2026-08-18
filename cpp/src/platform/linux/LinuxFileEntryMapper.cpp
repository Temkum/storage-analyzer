#include "system_analyzer/platform/linux/LinuxFileEntryMapper.hpp"

#include <system_error>

namespace system_analyzer::platform::linux
{

    domain::FileEntry LinuxFileEntryMapper::map(
        const std::filesystem::directory_entry &entry)
    {
        std::error_code error;

        domain::FileType type = domain::FileType::Other;

        if (entry.is_symlink(error))
        {
            type = domain::FileType::Symlink;
        }
        else if (entry.is_directory(error))
        {
            type = domain::FileType::Directory;
        }
        else if (entry.is_regular_file(error))
        {
            type = domain::FileType::File;
        }

        std::uintmax_t size = 0;

        if (type == domain::FileType::File)
        {
            size = entry.file_size(error);

            if (error)
            {
                size = 0;
            }
        }

        return domain::FileEntry{
            .path = entry.path(),
            .type = type,
            .size = size};
    }

} // namespace system_analyzer::platform::linux
