#include "system_analyzer/platform/windows/WindowsDiskUsageProvider.hpp"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <stdexcept>
#include <string>

namespace system_analyzer::platform::windows
{

    namespace
    {
        std::wstring driveRootFor(const std::filesystem::path &path)
        {
            std::error_code error;

            auto absolute = std::filesystem::absolute(path, error);

            if (error)
            {
                absolute = path;
            }

            std::wstring root = absolute.root_name().wstring();

            if (root.empty())
            {
                root = std::filesystem::current_path(error).root_name().wstring();
            }

            if (root.empty())
            {
                root = L"C:";
            }

            return root + L"\\";
        }
    }

    domain::DiskUsage WindowsDiskUsageProvider::getUsage(
        const std::filesystem::path &path) const
    {
        const std::wstring root = driveRootFor(path);

        ULARGE_INTEGER availableBytes{};
        ULARGE_INTEGER totalBytes{};
        ULARGE_INTEGER freeBytes{};

        if (GetDiskFreeSpaceExW(root.c_str(), &availableBytes, &totalBytes, &freeBytes) == 0)
        {
            throw std::runtime_error(
                "Failed to get filesystem statistics for \"" +
                path.string() + "\"");
        }

        const auto total =
            static_cast<std::uintmax_t>(totalBytes.QuadPart);

        const auto free =
            static_cast<std::uintmax_t>(freeBytes.QuadPart);

        const auto available =
            static_cast<std::uintmax_t>(availableBytes.QuadPart);

        const auto used = total > free ? total - free : 0;

        return {
            path,
            total,
            free,
            available,
            used};
    }

} // namespace system_analyzer::platform::windows