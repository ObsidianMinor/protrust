#ifndef PROTRUSTC_RUST_REPEATED_FIELD_GENERATOR_H__
#define PROTRUSTC_RUST_REPEATED_FIELD_GENERATOR_H__

#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_options.h>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

class RustRepeatedFieldGenerator : public RustFieldGenerator {
public:
    RustRepeatedFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);
    virtual ~RustRepeatedFieldGenerator();

    RustRepeatedFieldGenerator(const RustRepeatedFieldGenerator&) = delete;
    RustRepeatedFieldGenerator& operator=(const RustRepeatedFieldGenerator&) = delete;

    void GenerateMergeBranches(google::protobuf::io::Printer& printer) override;
    void GenerateCalculateSize(google::protobuf::io::Printer& printer) override;
    void GenerateWriteTo(google::protobuf::io::Printer& printer) override;
    void GenerateIsInitialized(google::protobuf::io::Printer& printer) override;
    void GenerateItems(google::protobuf::io::Printer& printer) override;

    virtual void GenerateExtension(google::protobuf::io::Printer& printer) override;

protected:
    virtual std::string GetFieldType() const override;
    /// Gets the generic arg 'T' for the field's RepeatedField impl
    virtual std::string GetImplGenericArg() const;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_REPEATED_FIELD_GENERATOR_H__