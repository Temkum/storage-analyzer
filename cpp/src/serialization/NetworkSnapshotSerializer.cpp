#include "system_analyzer/serialization/NetworkSnapshotSerializer.hpp"

#include <nlohmann/json.hpp>

namespace system_analyzer::serialization
{

    std::string NetworkSnapshotSerializer::toJson(
        const NetworkSnapshot &snapshot)
    {
        nlohmann::json json;

        json["timestamp"] = snapshot.timestamp;

        json["interfaces"] = nlohmann::json::array();

        for (const auto &interface : snapshot.interfaces)
        {
            json["interfaces"].push_back({{"id", interface.id},
                                          {"name", interface.name},
                                          {"bytesReceived",
                                           interface.bytesReceived},
                                          {"bytesSent", interface.bytesSent},
                                          {"isUp", interface.isUp}});
        }

        return json.dump();
    }

} // namespace system_analyzer::serialization