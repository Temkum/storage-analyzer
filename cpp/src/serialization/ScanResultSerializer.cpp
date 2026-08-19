#include "system_analyzer/serialization/ScanResultSerializer.hpp"

#include <nlohmann/json.hpp>

namespace system_analyzer::serialization
{

    std::string ScanResultSerializer::toJson(
        const domain::ScanResult &result)
    {
        nlohmann::json json;

        json["rootPath"] = result.rootPath.string();
        json["totalSize"] = result.totalSize;
        json["fileCount"] = result.fileCount;
        json["directoryCount"] = result.directoryCount;
        json["durationMs"] = result.durationMs;

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

        json["errors"] = nlohmann::json::array();

        for (const auto &error : result.errors)
        {
            json["errors"].push_back({{"path", error.path},
                                      {"message", error.message}});
        }

        return json.dump();
    }

} // namespace system_analyzer::serialization