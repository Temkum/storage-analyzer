#include "system_analyzer/platform/windows/WindowsVolumeProvider.hpp"

#include <windows.h>

#include <string>
#include <vector>

namespace system_analyzer::platform::windows
{

    namespace
    {
        std::string wideToUtf8(const std::wstring &value)
        {
            if (value.empty())
            {
                return {};
            }

            const auto size = WideCharToMultiByte(
                CP_UTF8,
                0,
                value.c_str(),
                static_cast<int>(value.size()),
                nullptr,
                0,
                nullptr,
                nullptr);

            std::string result(static_cast<std::size_t>(size), '\0');

            WideCharToMultiByte(
                CP_UTF8,
                0,
                value.c_str(),
                static_cast<int>(value.size()),
                result.data(),
                size,
                nullptr,
                nullptr);

            return result;
        }
    }

    std::vector<domain::MountedVolume> WindowsVolumeProvider::getVolumes() const
    {
        std::vector<domain::MountedVolume> volumes;

        const DWORD driveMask = GetLogicalDrives();

        if (driveMask == 0)
        {
            return volumes;
        }

        for (DWORD index = 0; index < 26; ++index)
        {
            if ((driveMask & (1u << index)) == 0)
            {
                continue;
            }

            const std::wstring mountPoint = {
                static_cast<wchar_t>(L'A' + index),
                L':',
                L'\\'};

            ULARGE_INTEGER availableBytes{};
            ULARGE_INTEGER totalBytes{};
            ULARGE_INTEGER freeBytes{};

            if (GetDiskFreeSpaceExW(
                    mountPoint.c_str(),
                    &availableBytes,
                    &totalBytes,
                    &freeBytes) == 0)
            {
                continue;
            }

            wchar_t filesystemName[MAX_PATH + 1] = {};
            DWORD volumeFlags = 0;

            // Drives that cannot be queried (empty optical bays, disconnected
            // network roots) are skipped, mirroring the Linux provider.
            if (GetVolumeInformationW(
                    mountPoint.c_str(),
                    nullptr,
                    0,
                    nullptr,
                    nullptr,
                    &volumeFlags,
                    filesystemName,
                    MAX_PATH) == 0)
            {
                continue;
            }

            const auto total =
                static_cast<std::uintmax_t>(totalBytes.QuadPart);

            const auto free =
                static_cast<std::uintmax_t>(freeBytes.QuadPart);

            const auto available =
                static_cast<std::uintmax_t>(availableBytes.QuadPart);

            if (total == 0)
            {
                continue;
            }

            const auto used = total > free ? total - free : 0;

            volumes.push_back({mountPoint,
                               wideToUtf8(filesystemName),
                               total,
                               free,
                               available,
                               used,
                               (volumeFlags & FILE_READ_ONLY_VOLUME) != 0});
        }

        return volumes;
    }

} // namespace system_analyzer::platform::windows