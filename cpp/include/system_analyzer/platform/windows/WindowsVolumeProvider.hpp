#pragma once

#include <vector>

#include "system_analyzer/core/IVolumeProvider.hpp"

namespace system_analyzer::platform::windows
{

    /// Enumerates fixed/removable volumes via the Win32 drive API, including
    /// filesystem name and read-only status.
    class WindowsVolumeProvider final : public core::IVolumeProvider
    {
    public:
        [[nodiscard]] std::vector<domain::MountedVolume> getVolumes() const override;
    };

} // namespace system_analyzer::platform::windows