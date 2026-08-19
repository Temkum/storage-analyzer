#pragma once

#include <string>

#include "system_analyzer/domain/ScanResult.hpp"

namespace system_analyzer::serialization {

class ScanResultSerializer {
public:
    [[nodiscard]] static std::string toJson(
        const domain::ScanResult& result
    );
};

} // namespace system_analyzer::serialization
