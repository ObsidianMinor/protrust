#ifndef PROTRUSTC_RUST_GLOBALS_GENERATOR_H__
#define PROTRUSTC_RUST_GLOBALS_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <vector>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustGlobalsGenerator : public RustSourceGenerator {
public:
    RustGlobalsGenerator(const Options& options);
    ~RustGlobalsGenerator();

    RustGlobalsGenerator(const RustGlobalsGenerator&) = delete;
    RustGlobalsGenerator& operator=(const RustGlobalsGenerator&) = delete;

    void Generate(const std::vector<const google::protobuf::FileDescriptor*>& files, google::protobuf::io::Printer& printer);
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_GLOBALS_GENERATOR_H__