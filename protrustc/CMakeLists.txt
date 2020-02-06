cmake_minimum_required (VERSION 3.8)

project(protrustc)

find_package(protobuf CONFIG REQUIRED)

set(files
    src/protrust/compiler/main.cpp
    src/protrust/compiler/rust_generator.h
    src/protrust/compiler/rust_generator.cpp
)

add_executable (protoc-gen-rust ${files})
target_include_directories(protoc-gen-rust PRIVATE src)
target_link_libraries(protoc-gen-rust PRIVATE protobuf::libprotoc protobuf::libprotobuf protobuf::libprotobuf-lite)