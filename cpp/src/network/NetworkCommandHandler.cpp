#include "system_analyzer/network/NetworkCommandHandler.hpp"

#include <nlohmann/json.hpp>

#include "system_analyzer/serialization/ApplicationNetworkSnapshotSerializer.hpp"
#include "system_analyzer/serialization/NetworkSnapshotSerializer.hpp"

namespace system_analyzer
{

    namespace
    {

        std::string errorResponse(
            const char *message)
        {
            nlohmann::json response;
            response["type"] = "error";
            response["message"] = message;
            return response.dump();
        }

    } // namespace

    NetworkCommandHandler::NetworkCommandHandler(
        INetworkUsageProvider &provider,
        IApplicationNetworkProvider &appProvider)
        : provider(provider), appProvider(appProvider)
    {
    }

    std::string NetworkCommandHandler::handle(
        const std::string &request)
    {
        // Responsibility is strictly: parse command -> call providers ->
        // serialize response. No sampling, deltas, SQLite or rate logic.
        nlohmann::json parsed;

        try
        {
            parsed = nlohmann::json::parse(request);
        }
        catch (const nlohmann::json::exception &)
        {
            return errorResponse("invalid json");
        }

        if (!parsed.is_object() ||
            !parsed.contains("command") ||
            !parsed["command"].is_string())
        {
            return errorResponse("invalid request");
        }

        const std::string command = parsed["command"].get<std::string>();

        if (command == "network_snapshot")
        {
            try
            {
                // The snapshot body comes from the contract serializers so
                // the NDJSON protocol and the schema contracts cannot drift
                // apart. A single request yields BOTH telemetry streams —
                // interfaces and applications — from one coherent observation
                // of the system, avoiding two IPC round-trips per second.
                nlohmann::json response = nlohmann::json::parse(
                    serialization::NetworkSnapshotSerializer::toJson(
                        provider.getSnapshot()));

                nlohmann::json appJson = nlohmann::json::parse(
                    serialization::ApplicationNetworkSnapshotSerializer::toJson(
                        appProvider.getSnapshot()));

                response["applications"] = appJson["applications"];
                response["type"] = "network_snapshot";

                return response.dump();
            }
            catch (const std::exception &)
            {
                return errorResponse("provider failure");
            }
        }

        if (command == "shutdown")
        {
            nlohmann::json response;
            response["type"] = "shutdown_ack";
            return response.dump();
        }

        return errorResponse("unknown command");
    }

} // namespace system_analyzer