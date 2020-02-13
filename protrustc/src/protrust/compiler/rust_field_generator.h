#ifndef PROTRUSTC_RUST_FIELD_GENERATOR_H__
#define PROTRUSTC_RUST_FIELD_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustFieldGenerator : public RustSourceGenerator {
public:
    RustFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);
    ~RustFieldGenerator();

    RustFieldGenerator(const RustFieldGenerator&) = delete;
    RustFieldGenerator& operator=(const RustFieldGenerator&) = delete;

    void GenerateExtension(google::protobuf::io::Printer& printer);
    void GenerateStructField(google::protobuf::io::Printer& printer);
    void GenerateMergeBranches(google::protobuf::io::Printer& printer);
    void GenerateItems(google::protobuf::io::Printer& printer);

private:
    const google::protobuf::FieldDescriptor* _field;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_FIELD_GENERATOR_H__