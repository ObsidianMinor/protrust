cmake_minimum_required (VERSION 3.8)

project(protrustc)

find_package(protobuf CONFIG REQUIRED)

set(files
    src/protrust/compiler/main.cpp
    src/protrust/compiler/rust_generator.h
    src/protrust/compiler/rust_generator.cpp
    src/protrust/compiler/rust_helpers.h
    src/protrust/compiler/rust_options.h
    src/protrust/compiler/rust_names.h
    src/protrust/compiler/rust_source_generator.h
    src/protrust/compiler/rust_source_generator.cpp
    src/protrust/compiler/rust_mod_generator.h
    src/protrust/compiler/rust_mod_generator.cpp
    src/protrust/compiler/rust_globals_generator.h
    src/protrust/compiler/rust_globals_generator.cpp
    src/protrust/compiler/rust_file_generator.h
    src/protrust/compiler/rust_file_generator.cpp
    src/protrust/compiler/rust_message_generator.h
    src/protrust/compiler/rust_message_generator.cpp
    src/protrust/compiler/rust_field_generator.h
    src/protrust/compiler/rust_field_generator.cpp
    src/protrust/compiler/rust_enum_generator.h
    src/protrust/compiler/rust_enum_generator.cpp
)

add_executable (protoc-gen-rust ${files})
target_include_directories(protoc-gen-rust PRIVATE src)
target_link_libraries(protoc-gen-rust PRIVATE protobuf::libprotoc protobuf::libprotobuf protobuf::libprotobuf-lite)