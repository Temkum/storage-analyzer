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

    result.diskUsage = {
        "/tmp",
        1000,
        400,
        350,
        600};

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

    assert(json["diskUsage"]["path"] == "/tmp");
    assert(json["diskUsage"]["totalBytes"] == 1000);
    assert(json["diskUsage"]["freeBytes"] == 400);
    assert(json["diskUsage"]["availableBytes"] == 350);
    assert(json["diskUsage"]["usedBytes"] == 600);

    return 0;
}
