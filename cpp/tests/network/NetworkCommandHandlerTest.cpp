#include <cassert>
#include <stdexcept>
#include <string>

#include <nlohmann/json.hpp>

#include "system_analyzer/network/NetworkCommandHandler.hpp"

namespace
{

    using system_analyzer::INetworkUsageProvider;
    using system_analyzer::NetworkCommandHandler;
    using system_analyzer::NetworkInterface;
    using system_analyzer::NetworkSnapshot;

    class FakeNetworkUsageProvider final : public INetworkUsageProvider
    {
    public:
        NetworkSnapshot getSnapshot() const override
        {
            return NetworkSnapshot{
                1700000000,
                {NetworkInterface{"fake0", "fake0", 10, 20, true}}};
        }
    };

    class FailingNetworkUsageProvider final : public INetworkUsageProvider
    {
    public:
        NetworkSnapshot getSnapshot() const override
        {
            throw std::runtime_error("provider exploded");
        }
    };

    nlohmann::json handleAndParse(
        NetworkCommandHandler &handler,
        const char *request)
    {
        return nlohmann::json::parse(handler.handle(request));
    }

} // namespace

int main()
{
    FakeNetworkUsageProvider fakeProvider;

    NetworkCommandHandler handler(fakeProvider);

    // network_snapshot returns a snapshot with the full contract shape.
    {
        const auto response = handleAndParse(
            handler, R"({"command":"network_snapshot"})");

        assert(response["type"] == "network_snapshot");
        assert(response["timestamp"] == 1700000000);

        assert(response["interfaces"].is_array());
        assert(response["interfaces"].size() == 1);

        const auto &interface = response["interfaces"][0];

        assert(interface["id"] == "fake0");
        assert(interface["name"] == "fake0");
        assert(interface["bytesReceived"] == 10);
        assert(interface["bytesSent"] == 20);
        assert(interface["isUp"] == true);
    }

    // Unknown commands become protocol errors, not crashes.
    {
        const auto response =
            handleAndParse(handler, R"({"command":"definitely_not_a_command"})");

        assert(response["type"] == "error");
        assert(response["message"] == "unknown command");
    }

    // Malformed JSON becomes a protocol error.
    {
        const auto response = handleAndParse(handler, R"({"command":)");

        assert(response["type"] == "error");
    }

    // Requests without a string "command" become protocol errors too.
    {
        const auto response = handleAndParse(handler, R"({"nope":1})");

        assert(response["type"] == "error");

        const auto nonStringCommand = handleAndParse(handler, R"({"command":42})");

        assert(nonStringCommand["type"] == "error");
    }

    // Shutdown is acknowledged with the fixed ack response.
    {
        const auto response =
            handleAndParse(handler, R"({"command":"shutdown"})");

        assert(response["type"] == "shutdown_ack");
    }

    // A failing provider surfaces as a protocol error instead of an escape.
    {
        FailingNetworkUsageProvider failingProvider;

        NetworkCommandHandler failingHandler(failingProvider);

        const auto response = handleAndParse(
            failingHandler, R"({"command":"network_snapshot"})");

        assert(response["type"] == "error");
    }

    return 0;
}