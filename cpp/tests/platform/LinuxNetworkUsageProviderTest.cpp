#include <cassert>
#include <iostream>

#include "system_analyzer/platform/linux/LinuxNetworkUsageProvider.hpp"

int main()
{
    using system_analyzer::LinuxNetworkUsageProvider;

    LinuxNetworkUsageProvider provider;

    const auto snapshot = provider.getSnapshot();

    assert(snapshot.timestamp > 0);

    // Deliberately no assumption about specific interface names: machines
    // report enp*, wlp*, br*, tun*, loopback, etc. Only shape is asserted.
    for (const auto &interface : snapshot.interfaces)
    {
        assert(!interface.id.empty());
        assert(!interface.name.empty());

        assert(!interface.isUp || interface.bytesSent >= 0);
    }

    std::cout << "Linux network provider test passed\n";

    return 0;
}