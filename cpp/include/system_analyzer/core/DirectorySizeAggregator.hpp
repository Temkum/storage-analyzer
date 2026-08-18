#pragma once

#include <cstdint>
#include <filesystem>
#include <unordered_map>

#include "system_analyzer/domain/FileEntry.hpp"

namespace system_analyzer::core {

class DirectorySizeAggregator {
public:
    void add(const domain::FileEntry& entry);

    [[nodiscard]] std::uintmax_t sizeOf(
        const std::filesystem::path& directory
    ) const;

private:
    std::unordered_map<std::string, std::uintmax_t> sizes_;
};

} // namespace system_analyzer::core
