#ifndef PROTRUSTC_RUST_HELPERS_H__
#define PROTRUSTC_RUST_HELPERS_H__

#include <string>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

std::string GetOutputFilePath(const google::protobuf::FileDescriptor* file, const char* import_name);

std::string GetFileDirPath(const google::protobuf::FileDescriptor* file);

std::string GetFileModName(const google::protobuf::FileDescriptor* file);

inline bool HasInnerItems(const google::protobuf::Descriptor* descriptor) {
    return
        descriptor->nested_type_count() != 0 ||
        descriptor->enum_type_count() != 0 ||
        descriptor->extension_count() != 0 ||
        descriptor->oneof_decl_count() != 0;
}

inline bool IsProto2File(const google::protobuf::FileDescriptor* descriptor) {
    return descriptor->syntax() == google::protobuf::FileDescriptor::SYNTAX_PROTO2;
}

std::string GetRawFieldType(const google::protobuf::FieldDescriptor* field);

std::string GetRustType(const google::protobuf::FieldDescriptor* field);
inline bool IsRustCopyable(const google::protobuf::FieldDescriptor* field) {
    switch (field->type()) {
        case google::protobuf::FieldDescriptor::TYPE_BYTES:
        case google::protobuf::FieldDescriptor::TYPE_GROUP:
        case google::protobuf::FieldDescriptor::TYPE_MESSAGE:
        case google::protobuf::FieldDescriptor::TYPE_STRING:
            return false;
        default:
            return true;
    }
}

std::string GetDefaultType(const google::protobuf::FieldDescriptor* field);
std::string GetDefaultTypeRef(const google::protobuf::FieldDescriptor* field);
std::string GetDefaultValue(const google::protobuf::FieldDescriptor* field);

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_HELPERS_H__