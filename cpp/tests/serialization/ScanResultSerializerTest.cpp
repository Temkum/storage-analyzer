#include <cassert>
#include <string>

#include <nlohmann/json.hpp>

#include "system_analyzer/domain/ScanResult.hpp"
#include "system_analyzer/serialization/ScanResultSerializer.hpp"

using system_analyzer::domain::FileEntry;
using system_analyzer::domain::FileType;
using system_analyzer::domain::ScanResult;
using system_analyzer::serialization::ScanResultSerializer;

int main()
{
    ScanResult result;

    result.entries.push_back({"/tmp/example.txt",
                              FileType::File,
                              128});

    result.directories.push_back({"/tmp",
                                  128});

    result.errors.push_back({"/restricted",
                             "Permission denied"});

    const std::string output =
        ScanResultSerializer::toJson(result);

    const auto json = nlohmann::json::parse(output);

    assert(json.contains("entries"));
    assert(json.contains("directories"));
    assert(json.contains("durationMs"));

    assert(json["entries"].size() == 1);
    assert(json["directories"].size() == 1);

    assert(json["entries"][0]["path"] == "/tmp/example.txt");
    assert(json["entries"][0]["size"] == 128);

    assert(json["directories"][0]["path"] == "/tmp");
    assert(json["directories"][0]["size"] == 128);

    return 0;
}
