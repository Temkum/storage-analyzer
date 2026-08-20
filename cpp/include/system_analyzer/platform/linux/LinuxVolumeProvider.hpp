#pragma once

#include "system_analyzer/core/IVolumeProvider.hpp"

namespace system_analyzer::platform::linux
{

    class LinuxVolumeProvider final : public core::IVolumeProvider
    {
    public:
        [[nodiscard]] std::vector<domain::MountedVolume> getVolumes() const override;
    };

} // namespace system_analyzer::platform::linux
