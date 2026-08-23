#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
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

    // The contract gate must fail even in Release builds, where assert() is
    // compiled out — otherwise Windows CI (which builds Release) would run
    // zero checks. Throw instead of asserting.
    [[noreturn]] void fail(
        const char *message)
    {
        throw std::runtime_error(message);
    }

    void require(
        bool condition,
        const char *message)
    {
        if (!condition)
        {
            fail(message);
        }
    }

    void requireUnsigned(const nlohmann::json &value)
    {
        require(value.is_number_unsigned(),
                "expected an unsigned integer");
    }

    void writeFile(
        const std::filesystem::path &path,
        const std::string &content)
    {
        std::ofstream stream(path);

        if (!stream.is_open())
        {
            fail("failed to create fixture file");
        }

        stream << content;
    }

} // namespace

void runContract();

int main()
{
    try
    {
        runContract();
        return 0;
    }
    catch (const std::exception &error)
    {
        std::cerr << "Scan schema contract violation: "
                  << error.what()
                  << '\n';
        return 1;
    }
}

void runContract()
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
    require(json.is_object(), "top level must be an object");

    for (const char *key : {"rootPath", "totalSize", "fileCount",
                            "directoryCount", "durationMs", "diskUsage",
                            "volumes", "entries", "directories", "errors"})
    {
        require(json.contains(key), "top level must contain key");
    }

    require(json["rootPath"].is_string(),
            "rootPath must be a string");
    requireUnsigned(json["totalSize"]);
    requireUnsigned(json["fileCount"]);
    requireUnsigned(json["directoryCount"]);
    requireUnsigned(json["durationMs"]);

    // Invariants that must hold on every platform ---------------------------
    require(json["totalSize"] >= 128 + 512 + 64,
            "totalSize must cover the fixture files");
    require(json["fileCount"] == 3,
            "fileCount must match the fixture");
    require(json["directoryCount"] == 1, // "nested"; root is not yielded
            "directoryCount must match the fixture");
    require(json["directories"].size() ==
                json["directoryCount"].get<unsigned>() + 1,
            "directories must include the root");

    // Entries ---------------------------------------------------------------
    require(json["entries"].is_array(), "entries must be an array");
    require(json["entries"].size() ==
                json["fileCount"].get<unsigned>() +
                    json["directoryCount"].get<unsigned>(),
            "entries must cover files and directories");

    for (const auto &entry : json["entries"])
    {
        require(entry.is_object(), "entry must be an object");
        require(entry.contains("path") && entry["path"].is_string(),
                "entry must have a string path");
        require(entry.contains("type") && entry["type"].is_number_unsigned(),
                "entry must have an unsigned type");
        require(entry.contains("size") && entry["size"].is_number_unsigned(),
                "entry must have an unsigned size");

        const auto type = entry["type"].get<unsigned>();
        require(type <= 3, "entry type outside enum domain"); // File, Directory, Symlink, Other
    }

    // Directories -----------------------------------------------------------
    require(json["directories"].is_array(), "directories must be an array");

    for (const auto &directory : json["directories"])
    {
        require(directory.is_object(), "directory must be an object");
        require(directory.contains("path") && directory["path"].is_string(),
                "directory must have a string path");
        require(directory.contains("size") && directory["size"].is_number_unsigned(),
                "directory must have an unsigned size");
    }

    // Errors ----------------------------------------------------------------
    require(json["errors"].is_array(), "errors must be an array");

    for (const auto &error : json["errors"])
    {
        require(error.is_object(), "error must be an object");
        require(error.contains("path") && error["path"].is_string(),
                "error must have a string path");
        require(error.contains("message") && error["message"].is_string(),
                "error must have a string message");
    }

    // Disk usage ------------------------------------------------------------
    require(json["diskUsage"].is_object(), "diskUsage must be an object");

    require(json["diskUsage"]["path"].is_string(),
            "diskUsage path must be a string");
    requireUnsigned(json["diskUsage"]["totalBytes"]);
    requireUnsigned(json["diskUsage"]["freeBytes"]);
    requireUnsigned(json["diskUsage"]["availableBytes"]);
    requireUnsigned(json["diskUsage"]["usedBytes"]);

    // Volumes ---------------------------------------------------------------
    require(json["volumes"].is_array(), "volumes must be an array");

    for (const auto &volume : json["volumes"])
    {
        require(volume.is_object(), "volume must be an object");

        for (const char *key : {"mountPoint", "filesystem", "totalBytes",
                                "freeBytes", "availableBytes", "usedBytes",
                                "readOnly"})
        {
            require(volume.contains(key), "volume must contain key");
        }

        require(volume["mountPoint"].is_string(),
                "volume mountPoint must be a string");
        require(volume["filesystem"].is_string(),
                "volume filesystem must be a string");
        requireUnsigned(volume["totalBytes"]);
        requireUnsigned(volume["freeBytes"]);
        requireUnsigned(volume["availableBytes"]);
        requireUnsigned(volume["usedBytes"]);
        require(volume["readOnly"].is_boolean(),
                "volume readOnly must be a boolean");
    }

    std::filesystem::remove_all(tempRoot, cleanupError);
}