#ifndef PROTRUSTC_RUST_ENUM_GENERATOR_H__
#define PROTRUSTC_RUST_ENUM_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustEnumGenerator : public RustSourceGenerator {
public:
    RustEnumGenerator(const google::protobuf::EnumDescriptor* enum_type, const Options& options);
    ~RustEnumGenerator();

    RustEnumGenerator(const RustEnumGenerator&) = delete;
    RustEnumGenerator& operator=(const RustEnumGenerator&) = delete;

    void Generate(google::protobuf::io::Printer& printer);

private:
    const google::protobuf::EnumDescriptor* _enum_type;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_ENUM_GENERATOR_H__