cmake_minimum_required(VERSION 3.20)

project(SystemAnalyzer
    VERSION 0.1.0
    DESCRIPTION "Cross-platform system resource analyzer"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

add_executable(system-analyzer
    cpp/src/main.cpp
    cpp/src/core/SystemInfo.cpp
)

target_include_directories(system-analyzer
    PRIVATE
        cpp/include
)