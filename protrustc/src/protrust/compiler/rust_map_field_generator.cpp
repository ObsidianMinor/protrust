#include <protrust/compiler/rust_map_field_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_helpers.h>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustMapFieldGenerator::RustMapFieldGenerator(const FieldDescriptor* field, const Options& options)
    : RustRepeatedFieldGenerator(field, options) { }

RustMapFieldGenerator::~RustMapFieldGenerator() { }

void RustMapFieldGenerator::GenerateExtension(io::Printer& printer) {
    // do nothing, map fields can't be extensions
}

const Descriptor* RustMapFieldGenerator::entry() const {
    return this->field()->message_type();
}

std::string RustMapFieldGenerator::GetFieldType() const {
    const FieldDescriptor* key_field = this->entry()->FindFieldByNumber(1);
    const FieldDescriptor* value_field = this->entry()->FindFieldByNumber(2);

    std::string key_type = GetRustType(key_field);
    std::string value_type = GetRustType(value_field);

    return "__prelude::MapField<" + key_type + ", " + value_type + ">";
}

std::string RustMapFieldGenerator::GetImplGenericArg() const {
    const FieldDescriptor* key_field = this->entry()->FindFieldByNumber(1);
    const FieldDescriptor* value_field = this->entry()->FindFieldByNumber(2);

    std::string key_type = GetRawFieldType(key_field);
    std::string value_type = GetRawFieldType(value_field);

    return "(" + key_type + ", " + value_type + ")";
}

} // compiler
} // protrust