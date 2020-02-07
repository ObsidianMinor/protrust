#ifndef PROTRUSTC_RUST_HELPERS_H__
#define PROTRUSTC_RUST_HELPERS_H__

#include <string>

#include <google/protobuf/descriptor.h>

namespace protrust {
namespace compiler {

std::string GetModPath(const google::protobuf::FileDescriptor* file, const char* import_name) {
    std::string result;
    // unimplemented
    return result;
}

std::string GetFileModName(const google::protobuf::FileDescriptor* file) {
    std::string result;
    // unimplemented
    return result;
}

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_HELPERS_H__