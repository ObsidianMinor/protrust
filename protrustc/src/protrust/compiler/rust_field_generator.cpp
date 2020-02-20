#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_repeated_field_generator.h>
#include <protrust/compiler/rust_map_field_generator.h>
#include <protrust/compiler/rust_message_field_generator.h>
#include <protrust/compiler/rust_primitive_field_generator.h>
#include <protrust/compiler/rust_source_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_names.h>

#include <memory>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustFieldGenerator::RustFieldGenerator(const FieldDescriptor* message, const Options& options)
    : RustSourceGenerator(options), _field(message) { }

RustFieldGenerator::~RustFieldGenerator() { }

void RustFieldGenerator::GenerateStructField(io::Printer& printer) {
    std::string field_type = this->GetFieldType();
    printer.Print(
        "$name$: $type$,\n",
        "name", GetFieldName(this->field()),
        "type", field_type
    );
}

void RustFieldGenerator::GenerateFieldNumberConst(io::Printer& printer) {
    printer.Print(
        "pub const $num$: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked($num_val$) };\n",
        "num", GetFieldNumberName(this->field()),
        "num_val", std::to_string(this->field()->number())
    );
}

const google::protobuf::FieldDescriptor* RustFieldGenerator::field() const {
    return this->_field;
}

std::unique_ptr<RustFieldGenerator> CreateFieldGenerator(const google::protobuf::FieldDescriptor* field, const Options& options) {
    if (field->is_repeated()) {
        if (field->is_map()) {
            return std::make_unique<RustMapFieldGenerator>(field, options);
        }
        else {
            return std::make_unique<RustRepeatedFieldGenerator>(field, options);
        }
    }
    else if (field->message_type() != NULL) {
        return std::make_unique<RustMessageFieldGenerator>(field, options);
    }
    else {
        return std::make_unique<RustPrimitiveFieldGenerator>(field, options);
    }
}

WIRE_TYPE GetWireType(FieldDescriptor::Type field_type) {
    switch (field_type) {
        case FieldDescriptor::Type::TYPE_FIXED64:
        case FieldDescriptor::Type::TYPE_SFIXED64:
        case FieldDescriptor::Type::TYPE_DOUBLE:
            return WIRE_TYPE::WT_BIT64;
        case FieldDescriptor::Type::TYPE_FIXED32:
        case FieldDescriptor::Type::TYPE_SFIXED32:
        case FieldDescriptor::Type::TYPE_FLOAT:
            return WIRE_TYPE::WT_BIT32;
        case FieldDescriptor::Type::TYPE_INT32:
        case FieldDescriptor::Type::TYPE_INT64:
        case FieldDescriptor::Type::TYPE_UINT32:
        case FieldDescriptor::Type::TYPE_UINT64:
        case FieldDescriptor::Type::TYPE_SINT32:
        case FieldDescriptor::Type::TYPE_SINT64:
        case FieldDescriptor::Type::TYPE_BOOL:
        case FieldDescriptor::Type::TYPE_ENUM:
            return WIRE_TYPE::WT_VARINT;
        case FieldDescriptor::Type::TYPE_MESSAGE:
        case FieldDescriptor::Type::TYPE_BYTES:
        case FieldDescriptor::Type::TYPE_STRING:
            return WIRE_TYPE::WT_LENGTH_DELIMITED;
        case FieldDescriptor::Type::TYPE_GROUP:
            return WIRE_TYPE::WT_START_GROUP;
        default:
            throw std::invalid_argument("unknown field type");
    };
};

uint MakeTag(int number, WIRE_TYPE wt) {
    return static_cast<uint>((number << 3) | wt);
};

} // compiler
} // protrust