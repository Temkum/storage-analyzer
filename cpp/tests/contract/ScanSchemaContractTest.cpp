#include <cassert>
#include <filesystem>
#include <fstream>
#include <string>

#include <nlohmann/json.hpp>

#include "system_analyzer/app/Application.hpp"
#include "system_analyzer/serialization/ScanResultSerializer.hpp"

/// Cross-platform JSON contract test.
///
/// Runs a real scan over a temporary fixture and asserts that the serialized
/// ScanResult keeps a stable schema on every platform: same fields, same
/// types, same enum domain, same error/volume semantics. Values differ per
/// machine; the shape may not.

using system_analyzer::app::Application;
using system_analyzer::domain::ScanResult;
using system_analyzer::serialization::ScanResultSerializer;

namespace
{

    void writeFile(
        const std::filesystem::path &path,
        const std::string &content)
    {
        std::ofstream stream(path);

        assert(stream.is_open());

        stream << content;
    }

    void assertUnsigned(const nlohmann::json &value)
    {
        assert(value.is_number_unsigned());
    }

} // namespace

int main()
{
    namespace fs = std::filesystem;

    const auto tempRoot =
        fs::temp_directory_path() / "sa-contract-test";

    std::error_code cleanupError;
    fs::remove_all(tempRoot, cleanupError);
    fs::create_directories(tempRoot / "nested", cleanupError);

    writeFile(tempRoot / "document.pdf", std::string(128, 'a'));
    writeFile(tempRoot / "nested" / "video.mp4", std::string(512, 'b'));
    writeFile(tempRoot / "nested" / "archive.zip", std::string(64, 'c'));

    const ScanResult result =
        Application{}.scan(tempRoot, {});

    const nlohmann::json json =
        nlohmann::json::parse(ScanResultSerializer::toJson(result));

    // Top-level schema ------------------------------------------------------
    assert(json.is_object());

    for (const char *key : {"rootPath", "totalSize", "fileCount",
                            "directoryCount", "durationMs", "diskUsage",
                            "volumes", "entries", "directories", "errors"})
    {
        assert(json.contains(key));
    }

    assert(json["rootPath"].is_string());
    assertUnsigned(json["totalSize"]);
    assertUnsigned(json["fileCount"]);
    assertUnsigned(json["directoryCount"]);
    assertUnsigned(json["durationMs"]);

    // Invariants that must hold on every platform ---------------------------
    assert(json["totalSize"] >= 128 + 512 + 64);
    assert(json["fileCount"] == 3);
    assert(json["directoryCount"] == 1); // "nested"; root is not yielded
    assert(json["directories"].size() ==
           json["directoryCount"].get<unsigned>() + 1);

    // Entries ---------------------------------------------------------------
    assert(json["entries"].is_array());
    assert(json["entries"].size() ==
           json["fileCount"].get<unsigned>() +
               json["directoryCount"].get<unsigned>());

    for (const auto &entry : json["entries"])
    {
        assert(entry.is_object());
        assert(entry.contains("path") && entry["path"].is_string());
        assert(entry.contains("type") && entry["type"].is_number_unsigned());
        assert(entry.contains("size") && entry["size"].is_number_unsigned());

        const auto type = entry["type"].get<unsigned>();
        assert(type <= 3); // File, Directory, Symlink, Other
    }

    // Directories -----------------------------------------------------------
    assert(json["directories"].is_array());

    for (const auto &directory : json["directories"])
    {
        assert(directory.is_object());
        assert(directory.contains("path") && directory["path"].is_string());
        assert(directory.contains("size") && directory["size"].is_number_unsigned());
    }

    // Errors ----------------------------------------------------------------
    assert(json["errors"].is_array());

    for (const auto &error : json["errors"])
    {
        assert(error.is_object());
        assert(error.contains("path") && error["path"].is_string());
        assert(error.contains("message") && error["message"].is_string());
    }

    // Disk usage ------------------------------------------------------------
    assert(json["diskUsage"].is_object());

    assert(json["diskUsage"]["path"].is_string());
    assertUnsigned(json["diskUsage"]["totalBytes"]);
    assertUnsigned(json["diskUsage"]["freeBytes"]);
    assertUnsigned(json["diskUsage"]["availableBytes"]);
    assertUnsigned(json["diskUsage"]["usedBytes"]);

    // Volumes ---------------------------------------------------------------
    assert(json["volumes"].is_array());

    for (const auto &volume : json["volumes"])
    {
        assert(volume.is_object());

        for (const char *key : {"mountPoint", "filesystem", "totalBytes",
                                "freeBytes", "availableBytes", "usedBytes",
                                "readOnly"})
        {
            assert(volume.contains(key));
        }

        assert(volume["mountPoint"].is_string());
        assert(volume["filesystem"].is_string());
        assertUnsigned(volume["totalBytes"]);
        assertUnsigned(volume["freeBytes"]);
        assertUnsigned(volume["availableBytes"]);
        assertUnsigned(volume["usedBytes"]);
        assert(volume["readOnly"].is_boolean());
    }

    std::filesystem::remove_all(tempRoot, cleanupError);

    return 0;
}