#ifndef PROTRUSTC_RUST_FIELD_GENERATOR_H__
#define PROTRUSTC_RUST_FIELD_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <memory>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustFieldGenerator : public RustSourceGenerator {
public:
    RustFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);
    virtual ~RustFieldGenerator();

    RustFieldGenerator(const RustFieldGenerator&) = delete;
    RustFieldGenerator& operator=(const RustFieldGenerator&) = delete;

    virtual void GenerateStructField(google::protobuf::io::Printer& printer);
    virtual void GenerateMergeBranches(google::protobuf::io::Printer& printer) = 0;
    virtual void GenerateCalculateSize(google::protobuf::io::Printer& printer) = 0;
    virtual void GenerateWriteTo(google::protobuf::io::Printer& printer) = 0;
    virtual void GenerateIsInitialized(google::protobuf::io::Printer& printer) = 0;
    virtual void GenerateFieldNumberConst(google::protobuf::io::Printer& printer);
    virtual void GenerateItems(google::protobuf::io::Printer& printer) = 0;
    virtual void GenerateExtension(google::protobuf::io::Printer& printer) = 0;

protected:
    /// Gets the field type
    virtual std::string GetFieldType() const = 0;

    const google::protobuf::FieldDescriptor* field() const;

private:
    const google::protobuf::FieldDescriptor* _field;
};

std::unique_ptr<RustFieldGenerator> CreateFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options);

enum WIRE_TYPE {
    WT_VARINT = 0,
    WT_BIT64 = 1,
    WT_LENGTH_DELIMITED = 2,
    WT_START_GROUP = 3,
    WT_END_GROUP = 4,
    WT_BIT32 = 5,
};

WIRE_TYPE GetWireType(google::protobuf::FieldDescriptor::Type field_type);

google::protobuf::uint MakeTag(int number, WIRE_TYPE wt);

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_FIELD_GENERATOR_H__