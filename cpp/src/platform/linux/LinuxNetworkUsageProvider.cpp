#include "system_analyzer/platform/linux/LinuxNetworkUsageProvider.hpp"

namespace system_analyzer
{

    NetworkSnapshot LinuxNetworkUsageProvider::getSnapshot() const
    {
        return NetworkSnapshot{};
    }

} // namespace system_analyzer