cmake_minimum_required(VERSION 3.10)
project(oxcc-cli LANGUAGES C)

add_executable(oxcc-cli main.c)

if (CMAKE_BUILD_TYPE MATCHES "^(Debug)$")
  target_link_directories(oxcc-cli PUBLIC "./target/debug")
else()
  target_link_directories(oxcc-cli PUBLIC "./target/release")
endif()

target_link_libraries(oxcc-cli PRIVATE oxcc m)