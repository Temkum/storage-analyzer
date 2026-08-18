#include "system_analyzer/core/DirectorySizeAggregator.hpp"

namespace system_analyzer::core {

void DirectorySizeAggregator::add(const domain::FileEntry& entry) {
    if (entry.type != domain::FileType::File) {
        return;
    }

    auto current = entry.path.parent_path();

    while (!current.empty()) {
        sizes_[current.string()] += entry.size;

        const auto parent = current.parent_path();

        if (parent == current) {
            break;
        }

        current = parent;
    }
}

std::uintmax_t DirectorySizeAggregator::sizeOf(
    const std::filesystem::path& directory
) const {
    const auto iterator = sizes_.find(directory.string());

    if (iterator == sizes_.end()) {
        return 0;
    }

    return iterator->second;
}

} // namespace system_analyzer::core
