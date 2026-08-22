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

        json["diskUsage"] = {
            {"path", result.diskUsage.path.string()},
            {"totalBytes", result.diskUsage.totalBytes},
            {"freeBytes", result.diskUsage.freeBytes},
            {"availableBytes", result.diskUsage.availableBytes},
            {"usedBytes", result.diskUsage.usedBytes}};

        json["volumes"] = nlohmann::json::array();

        for (const auto &volume : result.volumes)
        {
            json["volumes"].push_back({{"mountPoint", volume.mountPoint.string()},
                                       {"filesystem", volume.filesystem},
                                       {"totalBytes", volume.totalBytes},
                                       {"freeBytes", volume.freeBytes},
                                       {"availableBytes", volume.availableBytes},
                                       {"usedBytes", volume.usedBytes},
                                       {"readOnly", volume.readOnly}});
        }

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