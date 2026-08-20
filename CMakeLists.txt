cmake_minimum_required(VERSION 3.20)

project(SystemAnalyzer
    VERSION 0.1.0
    DESCRIPTION "Cross-platform system resource analyzer"
    LANGUAGES CXX
)

include(CTest)
enable_testing()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

add_executable(system-analyzer
    cpp/src/main.cpp
    cpp/src/app/Application.cpp
    cpp/src/core/SystemInfo.cpp
    cpp/src/core/DirectorySizeAggregator.cpp
    cpp/src/platform/linux/LinuxFileScanner.cpp
    cpp/src/platform/linux/LinuxFileEntryMapper.cpp
    cpp/src/platform/linux/LinuxDiskUsageProvider.cpp
    cpp/src/serialization/ScanResultSerializer.cpp
)

target_include_directories(system-analyzer
    PRIVATE
        cpp/include
)

# Stage the engine binary where Tauri expects it so `cargo tauri dev` always
# runs a sidecar that matches the freshly-built C++ code.
set(TAURI_BINARIES_DIR "${CMAKE_SOURCE_DIR}/apps/desktop/src-tauri/binaries")

add_custom_command(
    TARGET system-analyzer POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        $<TARGET_FILE:system-analyzer>
        "${TAURI_BINARIES_DIR}/system-analyzer-${CMAKE_HOST_SYSTEM_PROCESSOR}-unknown-linux-gnu"
    COMMENT "Staging sidecar binary for Tauri"
)

add_executable(directory-size-aggregator-test
    cpp/tests/core/DirectorySizeAggregatorTest.cpp
    cpp/src/core/DirectorySizeAggregator.cpp
)

target_include_directories(directory-size-aggregator-test
    PRIVATE
        cpp/include
)

add_executable(scan-result-test
    cpp/tests/core/ScanResultTest.cpp
)

target_include_directories(scan-result-test
    PRIVATE
        cpp/include
)

add_test(
    NAME scan-result
    COMMAND scan-result-test
)

add_test(
    NAME directory-size-aggregator
    COMMAND directory-size-aggregator-test
)

add_executable(scan-result-serializer-test
    cpp/tests/serialization/ScanResultSerializerTest.cpp
    cpp/src/serialization/ScanResultSerializer.cpp
)

target_include_directories(scan-result-serializer-test
    PRIVATE
        cpp/include
)

add_test(
    NAME scan-result-serializer
    COMMAND scan-result-serializer-test
)

add_executable(linux-file-scanner-test
    cpp/tests/platform/LinuxFileScannerTest.cpp
    cpp/src/platform/linux/LinuxFileScanner.cpp
    cpp/src/platform/linux/LinuxFileEntryMapper.cpp
)

target_include_directories(linux-file-scanner-test
    PRIVATE
        cpp/include
)

add_test(
    NAME linux-file-scanner
    COMMAND linux-file-scanner-test
)

add_executable(linux-disk-usage-test
    cpp/tests/platform/LinuxDiskUsageProviderTest.cpp
    cpp/src/platform/linux/LinuxDiskUsageProvider.cpp
)

target_include_directories(linux-disk-usage-test
    PRIVATE
        cpp/include
)

add_test(
    NAME linux-disk-usage
    COMMAND linux-disk-usage-test
)