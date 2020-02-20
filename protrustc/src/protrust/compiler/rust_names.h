#ifndef PROTRUSTC_RUST_NAMES_H__
#define PROTRUSTC_RUST_NAMES_H__

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

std::string Escape(std::string str);

// messages
inline std::string GetMessageName(const google::protobuf::Descriptor* descriptor) {
    return Escape(descriptor->name());
}
std::string GetMessageModName(const google::protobuf::Descriptor* descriptor);

// enums
inline std::string GetEnumName(const google::protobuf::EnumDescriptor* descriptor) {
    return Escape(descriptor->name());
}
std::string GetEnumValueName(const google::protobuf::EnumValueDescriptor* descriptor);

// fields
inline std::string GetFieldName(const google::protobuf::FieldDescriptor* descriptor) {
    return Escape(descriptor->name());
}
std::string GetFieldNumberName(const google::protobuf::FieldDescriptor* descriptor);
std::string GetFieldDefaultName(const google::protobuf::FieldDescriptor* descriptor);

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_NAMES_H__
