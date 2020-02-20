#ifndef PROTRUSTC_RUST_MAP_FIELD_GENERATOR_H__
#define PROTRUSTC_RUST_MAP_FIELD_GENERATOR_H__

#include <protrust/compiler/rust_repeated_field_generator.h>
#include <protrust/compiler/rust_options.h>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

class RustMapFieldGenerator : public RustRepeatedFieldGenerator {
public:
    RustMapFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);
    ~RustMapFieldGenerator();

    RustMapFieldGenerator(const RustMapFieldGenerator&) = delete;
    RustMapFieldGenerator& operator=(const RustMapFieldGenerator&) = delete;

    void GenerateExtension(google::protobuf::io::Printer& printer) override;

protected:
    std::string GetFieldType() const override;
    std::string GetImplGenericArg() const override;

private:
    const google::protobuf::Descriptor* entry() const;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_MAP_FIELD_GENERATOR_H__