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
)

target_include_directories(system-analyzer
    PRIVATE
        cpp/include
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