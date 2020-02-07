#include <protrust/compiler/rust_file_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustFileGenerator::RustFileGenerator(const FileDescriptor* file, const Options& options)
    : RustSourceGenerator(options), _file(file) { }

RustFileGenerator::~RustFileGenerator() { }

void RustFileGenerator::Generate(io::Printer& printer) {
    // unimplemented
}

} // compiler
} // protrust