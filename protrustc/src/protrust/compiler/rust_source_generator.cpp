#include <protrust/compiler/rust_source_generator.h>
#include <protrust/compiler/rust_options.h>

namespace protrust {
namespace compiler {

RustSourceGenerator::RustSourceGenerator(const Options& options) : options_(options) { }

RustSourceGenerator::~RustSourceGenerator() { }

const Options& RustSourceGenerator::options() {
    return this->options_;
}

} // compiler
} // protrust