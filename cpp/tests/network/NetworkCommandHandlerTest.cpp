#include <cassert>
#include <stdexcept>
#include <string>

#include <nlohmann/json.hpp>

#include "system_analyzer/domain/ApplicationNetworkUsage.hpp"
#include "system_analyzer/network/NetworkCommandHandler.hpp"
#include "system_analyzer/platform/application_network_provider.hpp"

namespace
{

    using system_analyzer::ApplicationNetworkSnapshot;
    using system_analyzer::ApplicationNetworkUsage;
    using system_analyzer::IApplicationNetworkProvider;
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

    class FakeApplicationNetworkProvider final : public IApplicationNetworkProvider
    {
    public:
        ApplicationNetworkSnapshot getSnapshot() override
        {
            return ApplicationNetworkSnapshot{
                1700000000,
                {ApplicationNetworkUsage{
                    "/usr/bin/fake-app", "fake-app", "/usr/bin/fake-app",
                    1000, 500}}};
        }
    };

    class EmptyApplicationNetworkProvider final : public IApplicationNetworkProvider
    {
    public:
        ApplicationNetworkSnapshot getSnapshot() override
        {
            return ApplicationNetworkSnapshot{1700000000, {}};
        }
    };

    class FailingApplicationNetworkProvider final : public IApplicationNetworkProvider
    {
    public:
        ApplicationNetworkSnapshot getSnapshot() override
        {
            throw std::runtime_error("app provider exploded");
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
    FakeApplicationNetworkProvider fakeAppProvider;

    NetworkCommandHandler handler(fakeProvider, fakeAppProvider);

    // Combined network_snapshot returns interfaces AND applications.
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

        assert(response.contains("applications"));
        assert(response["applications"].is_array());
        assert(response["applications"].size() == 1);

        const auto &app = response["applications"][0];
        assert(app["appId"] == "/usr/bin/fake-app");
        assert(app["processName"] == "fake-app");
        assert(app["executablePath"] == "/usr/bin/fake-app");
        assert(app["bytesReceived"] == 1000);
        assert(app["bytesSent"] == 500);
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

    // A failing interface provider surfaces as a protocol error.
    {
        FailingNetworkUsageProvider failingProvider;
        FakeApplicationNetworkProvider fakeApp;

        NetworkCommandHandler failingHandler(failingProvider, fakeApp);

        const auto response = handleAndParse(
            failingHandler, R"({"command":"network_snapshot"})");

        assert(response["type"] == "error");
    }

    // A failing application provider surfaces as a protocol error.
    {
        FakeNetworkUsageProvider fakeNet;
        FailingApplicationNetworkProvider failingAppProvider;

        NetworkCommandHandler failingHandler(fakeNet, failingAppProvider);

        const auto response = handleAndParse(
            failingHandler, R"({"command":"network_snapshot"})");

        assert(response["type"] == "error");
    }

    // An empty applications array is a valid combined snapshot (no
    // attributable TCP processes does NOT mean network provider failed).
    {
        FakeNetworkUsageProvider fakeNet;
        EmptyApplicationNetworkProvider emptyApp;

        NetworkCommandHandler handler(fakeNet, emptyApp);

        const auto response = handleAndParse(
            handler, R"({"command":"network_snapshot"})");

        assert(response["type"] == "network_snapshot");
        assert(response["applications"].is_array());
        assert(response["applications"].empty());
    }

    return 0;
}
