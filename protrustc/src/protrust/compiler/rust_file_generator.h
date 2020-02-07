#ifndef PROTRUSTC_RUST_FILE_GENERATOR_H__
#define PROTRUSTC_RUST_FILE_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustFileGenerator : public RustSourceGenerator {
public:
    RustFileGenerator(const google::protobuf::FileDescriptor* file, const Options& options);
    ~RustFileGenerator();

    RustFileGenerator(const RustFileGenerator&) = delete;
    RustFileGenerator& operator=(const RustFileGenerator&) = delete;

    void Generate(google::protobuf::io::Printer& printer);

private:
    const google::protobuf::FileDescriptor* _file;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_FILE_GENERATOR_H__