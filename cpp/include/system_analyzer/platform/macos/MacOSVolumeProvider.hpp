#pragma once

#include <vector>

#include "system_analyzer/core/IVolumeProvider.hpp"

namespace system_analyzer::platform::macos
{

    /// Enumerates mounted volumes via getmntinfo (BSD), including filesystem
    /// type and read-only status from mount flags.
    class MacOSVolumeProvider final : public core::IVolumeProvider
    {
    public:
        [[nodiscard]] std::vector<domain::MountedVolume> getVolumes() const override;
    };

} // namespace system_analyzer::platform::macos