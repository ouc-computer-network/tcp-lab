cmake_minimum_required(VERSION 3.20)
project(tcp_lab_cpp_sdk LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(rdt1_sender SHARED src/rdt1_sender.cpp)
target_include_directories(rdt1_sender PRIVATE include)
set_target_properties(rdt1_sender PROPERTIES OUTPUT_NAME "rdt1_sender")

add_library(rdt1_receiver SHARED src/rdt1_receiver.cpp)
target_include_directories(rdt1_receiver PRIVATE include)
set_target_properties(rdt1_receiver PROPERTIES OUTPUT_NAME "rdt1_receiver")
