#ifndef PROTRUSTC_RUST_PRIMITIVE_FIELD_GENERATOR_H__
#define PROTRUSTC_RUST_PRIMITIVE_FIELD_GENERATOR_H__

#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_options.h>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

class RustPrimitiveFieldGenerator : public RustFieldGenerator {
public:
    RustPrimitiveFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);
    ~RustPrimitiveFieldGenerator();

    RustPrimitiveFieldGenerator(const RustPrimitiveFieldGenerator&) = delete;
    RustPrimitiveFieldGenerator& operator=(const RustPrimitiveFieldGenerator&) = delete;

    void GenerateMergeBranches(google::protobuf::io::Printer& printer) override;
    void GenerateCalculateSize(google::protobuf::io::Printer& printer) override;
    void GenerateWriteTo(google::protobuf::io::Printer& printer) override;
    void GenerateIsInitialized(google::protobuf::io::Printer& printer) override;
    void GenerateItems(google::protobuf::io::Printer& printer) override;
    void GenerateExtension(google::protobuf::io::Printer& printer) override;

protected:
    std::string GetFieldType() const override;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_PRIMITIVE_FIELD_GENERATOR_H__