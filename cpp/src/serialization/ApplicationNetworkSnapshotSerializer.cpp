#include "system_analyzer/serialization/ApplicationNetworkSnapshotSerializer.hpp"

#include <nlohmann/json.hpp>

namespace system_analyzer::serialization
{

    std::string ApplicationNetworkSnapshotSerializer::toJson(
        const ApplicationNetworkSnapshot &snapshot)
    {
        nlohmann::json json;

        json["timestamp"] = snapshot.timestamp;
        json["applications"] = nlohmann::json::array();

        for (const auto &app : snapshot.applications)
        {
            json["applications"].push_back(
                {{"appId", app.appId},
                 {"processName", app.processName},
                 {"executablePath", app.executablePath},
                 {"bytesReceived", app.bytesReceived},
                 {"bytesSent", app.bytesSent}});
        }

        return json.dump();
    }

} // namespace system_analyzer::serialization