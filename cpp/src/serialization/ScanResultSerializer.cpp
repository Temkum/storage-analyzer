#include "system_analyzer/serialization/ScanResultSerializer.hpp"

#include <sstream>

namespace system_analyzer::serialization {

std::string ScanResultSerializer::toJson(
    const domain::ScanResult& result
) {
    std::ostringstream json;

    json << "{";
    json << "\"entries\":[";

    for (std::size_t i = 0; i < result.entries.size(); ++i) {
        const auto& entry = result.entries[i];

        if (i > 0) {
            json << ",";
        }

        json << "{";
        json << "\"path\":\"" << entry.path.string() << "\",";
        json << "\"size\":" << entry.size;
        json << "}";
    }

    json << "],";

    json << "\"directories\":[";

    for (std::size_t i = 0; i < result.directories.size(); ++i) {
        const auto& directory = result.directories[i];

        if (i > 0) {
            json << ",";
        }

        json << "{";
        json << "\"path\":\"" << directory.path.string() << "\",";
        json << "\"size\":" << directory.size;
        json << "}";
    }

    json << "]";

    json << "}";

    return json.str();
}

} // namespace system_analyzer::serialization
