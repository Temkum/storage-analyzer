#include <cassert>
#include <filesystem>

#include "system_analyzer/core/DirectorySizeAggregator.hpp"

using system_analyzer::core::DirectorySizeAggregator;
using system_analyzer::domain::FileEntry;
using system_analyzer::domain::FileType;

int main() {
    const std::filesystem::path root = "/tmp/test-project";
    const std::filesystem::path src = root / "src";

    DirectorySizeAggregator aggregator;

    aggregator.add({
        root / "README.md",
        FileType::File,
        100
    });

    aggregator.add({
        src / "main.cpp",
        FileType::File,
        200
    });

    aggregator.add({
        src / "utils.cpp",
        FileType::File,
        300
    });

    assert(aggregator.sizeOf(root) == 600);
    assert(aggregator.sizeOf(src) == 500);

    aggregator.add({
        root / "src",
        FileType::Directory,
        0
    });

    assert(aggregator.sizeOf(root) == 600);
    assert(aggregator.sizeOf(src) == 500);

    return 0;
}
