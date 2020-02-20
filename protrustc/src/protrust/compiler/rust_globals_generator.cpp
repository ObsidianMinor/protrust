#include <protrust/compiler/rust_globals_generator.h>
#include <protrust/compiler/rust_source_generator.h>
#include <protrust/compiler/rust_options.h>

#include <vector>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustGlobalsGenerator::RustGlobalsGenerator(const Options& options)
    : RustSourceGenerator(options) { }

RustGlobalsGenerator::~RustGlobalsGenerator() { }

void RustGlobalsGenerator::Generate(const std::vector<const FileDescriptor*>& files, io::Printer& printer) {
    // do nothing for now, we don't have reflection facilities set up yet
}

} // compiler
} // protrust