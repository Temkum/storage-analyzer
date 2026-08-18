#include <iostream>

#include "system_analyzer/core/SystemInfo.hpp"

int main()
{
    const system_analyzer::core::SystemInfo system("System Analyzer");

    std::cout << system.name() << " C++ engine starting...\n";

    return 0;
}