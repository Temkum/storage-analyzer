#include "system_analyzer/serialization/ScanResultSerializer.hpp"

#include <nlohmann/json.hpp>

namespace system_analyzer::serialization
{

    std::string ScanResultSerializer::toJson(
        const domain::ScanResult &result)
    {
        nlohmann::json json;

        json["entries"] = nlohmann::json::array();

        for (const auto &entry : result.entries)
        {
            json["entries"].push_back({{"path", entry.path.string()},
                                       {"type", static_cast<int>(entry.type)},
                                       {"size", entry.size}});
        }

        json["directories"] = nlohmann::json::array();

        for (const auto &directory : result.directories)
        {
            json["directories"].push_back({{"path", directory.path.string()},
                                           {"size", directory.size}});
        }

        return json.dump();
    }

} // namespace system_analyzer::serialization