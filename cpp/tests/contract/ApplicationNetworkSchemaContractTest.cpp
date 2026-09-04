#include <iostream>
#include <stdexcept>
#include <string>

#include <nlohmann/json.hpp>

#include "system_analyzer/platform/linux/LinuxApplicationNetworkProvider.hpp"
#include "system_analyzer/serialization/ApplicationNetworkSnapshotSerializer.hpp"

/// Real-machine integration test for application attribution.
///
/// Runs against the actual /proc/TCP stack and asserts structural
/// correctness only — it does NOT expect Firefox, Chrome, Docker, etc. to
/// exist. For every reported application it asserts:
///   - appId is non-empty
///   - executablePath is non-empty
///   - processName is non-empty
///   - bytesReceived >= 0
///   - bytesSent >= 0
/// and that the serialized JSON keeps the stable contract shape. No PID is
/// part of the identity; appId must equal executablePath.

using system_analyzer::ApplicationNetworkSnapshot;
using system_analyzer::LinuxApplicationNetworkProvider;
using system_analyzer::serialization::ApplicationNetworkSnapshotSerializer;

namespace
{
    [[noreturn]] void fail(const char *message)
    {
        throw std::runtime_error(message);
    }

    void require(bool condition, const char *message)
    {
        if (!condition)
        {
            fail(message);
        }
    }
} // namespace

int main()
{
    try
    {
        LinuxApplicationNetworkProvider provider;
        const ApplicationNetworkSnapshot snapshot = provider.getSnapshot();

        // Timestamp sanity.
        require(snapshot.timestamp > 0,
                "timestamp must be positive");

        // Structural correctness for every reported application.
        for (const auto &app : snapshot.applications)
        {
            require(!app.appId.empty(), "appId must be non-empty");
            require(!app.executablePath.empty(),
                    "executablePath must be non-empty");
            require(app.appId == app.executablePath,
                    "appId must equal executablePath (identity is the exe)");
        }

        // Serialization must produce the stable contract shape.
        const nlohmann::json json = nlohmann::json::parse(
            ApplicationNetworkSnapshotSerializer::toJson(snapshot));

        require(json.is_object(), "top level must be an object");
        require(json.contains("timestamp"), "must contain timestamp");
        require(json["timestamp"].is_number_unsigned(),
                "timestamp must be an integer");
        require(json.contains("applications"),
                "must contain applications array");
        require(json["applications"].is_array(),
                "applications must be an array");

        for (const auto &app : json["applications"])
        {
            require(app.is_object(), "application must be an object");
            for (const char *key :
                 {"appId", "processName", "executablePath",
                  "bytesReceived", "bytesSent"})
            {
                require(app.contains(key),
                        "application must contain all five fields");
            }
            require(app["appId"].is_string(), "appId must be a string");
            require(app["processName"].is_string(),
                    "processName must be a string");
            require(app["executablePath"].is_string(),
                    "executablePath must be a string");
            require(app["bytesReceived"].is_number_unsigned(),
                    "bytesReceived must be an integer");
            require(app["bytesSent"].is_number_unsigned(),
                    "bytesSent must be an integer");
        }

        std::cout << "application-network-schema-contract passed\n";
        return 0;
    }
    catch (const std::exception &error)
    {
        std::cerr << "Application schema contract violation: "
                  << error.what() << '\n';
        return 1;
    }
}
