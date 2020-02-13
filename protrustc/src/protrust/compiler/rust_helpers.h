#ifndef PROTRUSTC_RUST_HELPERS_H__
#define PROTRUSTC_RUST_HELPERS_H__

#include <string>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

std::string GetOutputFilePath(const google::protobuf::FileDescriptor* file, const char* import_name) {
    std::string result;
    // unimplemented
    return result;
}

std::string GetFileDirPath(const google::protobuf::FileDescriptor* file) {
    std::string result;
    // unimplemented
    return result;
}

std::string GetFileModName(const google::protobuf::FileDescriptor* file) {
    std::string result;
    // unimplemented
    return result;
}

bool HasInnerItems(const google::protobuf::Descriptor* descriptor) {
    return 
        descriptor->nested_type_count() != 0 || 
        descriptor->enum_type_count() != 0 ||
        descriptor->extension_count() != 0;
}

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_HELPERS_H__