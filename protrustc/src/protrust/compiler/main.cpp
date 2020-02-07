#include <protrust/compiler/rust_generator.h>
#include <google/protobuf/compiler/plugin.h>

int main(int argc, char** argv) {
    protrust::compiler::RustGenerator generator;
    return google::protobuf::compiler::PluginMain(argc, argv, &generator);
}
