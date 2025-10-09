cmake_minimum_required(VERSION 3.14)
project(CalculatorMutationTests)

# C++17 standard (required for modern C++ features)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Enable testing
enable_testing()

# Fetch Google Test framework
include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/refs/tags/release-1.12.1.zip
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)

# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)

FetchContent_MakeAvailable(googletest)

# Add calculator library
add_library(calculator STATIC calculator.cpp)
target_include_directories(calculator PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

# Add test executable
add_executable(test_calculator test_calculator.cpp)
target_link_libraries(test_calculator
    PRIVATE
    calculator
    gtest_main
)

# Discover and register tests
include(GoogleTest)
gtest_discover_tests(test_calculator)

# Add a custom target to run tests
add_custom_target(run_tests
    COMMAND ${CMAKE_CTEST_COMMAND} --output-on-failure
    DEPENDS test_calculator
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running mutation testing suite..."
)
