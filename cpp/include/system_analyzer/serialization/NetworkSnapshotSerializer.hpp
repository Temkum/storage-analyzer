#pragma once

#include <string>

#include "system_analyzer/domain/Network.hpp"

namespace system_analyzer::serialization {

class NetworkSnapshotSerializer {
public:
    // Network domain types intentionally live in the top-level
    // system_analyzer namespace (no nested domain:: qualifier).
    [[nodiscard]] static std::string toJson(
        const NetworkSnapshot& snapshot
    );
};

} // namespace system_analyzer::serialization