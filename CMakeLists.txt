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

# Platform selection
# The sidecar is built natively per OS (no C++ cross-compilation). Each
# platform owns a directory under cpp/src/platform/<os> containing its
# providers plus a PlatformFactory.cpp that implements the factory declared
# in cpp/include/system_analyzer/platform/factory.hpp.

if(WIN32)
    set(SYSTEM_ANALYZER_PLATFORM_DIR windows)
    set(SIDECAR_TARGET_TRIPLE "x86_64-pc-windows-msvc" CACHE STRING
        "Tauri sidecar target triple")
    set(SIDECAR_FILENAME "system-analyzer-${SIDECAR_TARGET_TRIPLE}.exe")
elseif(APPLE)
    set(SYSTEM_ANALYZER_PLATFORM_DIR macos)
    set(SIDECAR_TARGET_TRIPLE "x86_64-apple-darwin" CACHE STRING
        "Tauri sidecar target triple")
    set(SIDECAR_FILENAME "system-analyzer-${SIDECAR_TARGET_TRIPLE}")
else()
    set(SYSTEM_ANALYZER_PLATFORM_DIR linux)
    set(SIDECAR_TARGET_TRIPLE "${CMAKE_HOST_SYSTEM_PROCESSOR}-unknown-linux-gnu" CACHE STRING
        "Tauri sidecar target triple")
    set(SIDECAR_FILENAME "system-analyzer-${SIDECAR_TARGET_TRIPLE}")
endif()

message(STATUS "Platform layer: ${SYSTEM_ANALYZER_PLATFORM_DIR} (sidecar: ${SIDECAR_FILENAME})")

set(PLATFORM_SOURCES
    cpp/src/platform/${SYSTEM_ANALYZER_PLATFORM_DIR}/PlatformFactory.cpp
)

if(WIN32)
    list(APPEND PLATFORM_SOURCES
        cpp/src/platform/windows/WindowsVolumeProvider.cpp
        cpp/src/platform/windows/WindowsDiskUsageProvider.cpp
    )
elseif(APPLE)
    list(APPEND PLATFORM_SOURCES
        cpp/src/platform/macos/MacOSVolumeProvider.cpp
        cpp/src/platform/macos/MacOSDiskUsageProvider.cpp
    )
else()
    list(APPEND PLATFORM_SOURCES
        cpp/src/platform/linux/LinuxVolumeProvider.cpp
        cpp/src/platform/linux/LinuxDiskUsageProvider.cpp
    )
endif()

# Engine

set(ENGINE_SOURCES
    cpp/src/app/Application.cpp
    cpp/src/core/SystemInfo.cpp
    cpp/src/core/DirectorySizeAggregator.cpp
    cpp/src/serialization/ScanResultSerializer.cpp
    ${PLATFORM_SOURCES}
)

add_executable(system-analyzer
    cpp/src/main.cpp
    ${ENGINE_SOURCES}
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
        "${TAURI_BINARIES_DIR}/${SIDECAR_FILENAME}"
    COMMENT "Staging sidecar binary for Tauri (${SIDECAR_FILENAME})"
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

add_executable(linux-volume-provider-test
    cpp/tests/platform/LinuxVolumeProviderTest.cpp
    cpp/src/platform/linux/LinuxVolumeProvider.cpp
)

target_include_directories(linux-volume-provider-test
    PRIVATE
        cpp/include
)

add_test(
    NAME linux-volume-provider
    COMMAND linux-volume-provider-test
)

# Cross-platform contract test
#
# Runs a real scan through the Application and asserts the serialized
# ScanResult keeps a stable JSON schema on every platform. This is the gate
# that keeps Linux/Windows/macOS sidecars wire-compatible with the UI.

add_executable(scan-schema-contract-test
    cpp/tests/contract/ScanSchemaContractTest.cpp
    ${ENGINE_SOURCES}
)

target_include_directories(scan-schema-contract-test
    PRIVATE
        cpp/include
)

add_test(
    NAME scan-schema-contract
    COMMAND scan-schema-contract-test
)