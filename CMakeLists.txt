cmake_minimum_required(VERSION 3.20)

project(SystemAnalyzer
    VERSION 0.1.0
    DESCRIPTION "Cross-platform system resource analyzer"
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_executable(system-analyzer
    cpp/src/main.cpp
)