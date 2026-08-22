#pragma once

#include <vector>

#include "system_analyzer/domain/MountedVolume.hpp"

namespace system_analyzer::core
{

    class IVolumeProvider
    {
    public:
        virtual ~IVolumeProvider() = default;

        [[nodiscard]] virtual std::vector<domain::MountedVolume> getVolumes() const = 0;
    };

} // namespace system_analyzer::core
