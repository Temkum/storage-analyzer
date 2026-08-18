#include <cassert>

#include "system_analyzer/domain/ScanResult.hpp"

using system_analyzer::domain::DirectorySize;
using system_analyzer::domain::FileEntry;
using system_analyzer::domain::FileType;
using system_analyzer::domain::ScanResult;

int main()
{
    ScanResult result;

    result.entries.push_back({"/tmp/example.txt",
                              FileType::File,
                              100});

    result.directories.push_back({"/tmp",
                                  100});

    assert(result.entries.size() == 1);
    assert(result.entries[0].size == 100);

    assert(result.directories.size() == 1);
    assert(result.directories[0].size == 100);

    return 0;
}
