#include <iostream>
#include <stdexcept>

#include <nlohmann/json.hpp>

#include "system_analyzer/platform/factory.hpp"
#include "system_analyzer/serialization/NetworkSnapshotSerializer.hpp"

/// Cross-platform JSON contract test for the network domain.
///
/// Builds the provider through the platform factory (no #ifdef here — the
/// per-platform PlatformFactory selects the implementation), takes a real
/// snapshot, serializes it and asserts the stable schema that every future
/// provider (Windows, macOS) must produce before it reaches the Tauri layer
/// and the Vue components. Values differ per machine; the shape may not.

using system_analyzer::NetworkSnapshot;
using system_analyzer::platform::createNetworkUsageProvider;
using system_analyzer::serialization::NetworkSnapshotSerializer;

namespace
{

    // The contract gate must fail even in Release builds, where assert() is
    // compiled out — otherwise Windows CI (which builds Release) would run
    // zero checks. Throw instead of asserting.
    [[noreturn]] void fail(
        const char *message)
    {
        throw std::runtime_error(message);
    }

    void require(
        bool condition,
        const char *message)
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
        const auto provider = createNetworkUsageProvider();

        require(provider != nullptr,
                "createNetworkUsageProvider must return a provider");

        const NetworkSnapshot snapshot = provider->getSnapshot();

        const nlohmann::json json =
            nlohmann::json::parse(NetworkSnapshotSerializer::toJson(snapshot));

        // Top-level schema ------------------------------------------------------
        require(json.is_object(), "top level must be an object");

        for (const char *key : {"timestamp", "interfaces"})
        {
            require(json.contains(key), "top level must contain key");
        }

        require(json["timestamp"].is_number_unsigned(),
                "timestamp must be an integer");
        require(json["interfaces"].is_array(),
                "interfaces must be an array");

        // Interface objects -----------------------------------------------------
        for (const auto &interface : json["interfaces"])
        {
            require(interface.is_object(), "interface must be an object");

            for (const char *key : {"id", "name", "bytesReceived",
                                    "bytesSent", "isUp"})
            {
                require(interface.contains(key),
                        "interface must contain all five required fields");
            }

            require(interface["id"].is_string(),
                    "interface id must be a string");
            require(interface["name"].is_string(),
                    "interface name must be a string");
            require(interface["bytesReceived"].is_number_unsigned(),
                    "interface bytesReceived must be an integer");
            require(interface["bytesSent"].is_number_unsigned(),
                    "interface bytesSent must be an integer");
            require(interface["isUp"].is_boolean(),
                    "interface isUp must be a boolean");
        }

        std::cout << "network-schema-contract passed\n";
        return 0;
    }
    catch (const std::exception &error)
    {
        std::cerr << "Network schema contract violation: "
                  << error.what()
                  << '\n';
        return 1;
    }
}