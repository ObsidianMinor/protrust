#ifndef PROTRUSTC_RUST_SOURCE_GENERATOR_H__
#define PROTRUSTC_RUST_SOURCE_GENERATOR_H__

#include <protrust/compiler/rust_options.h>

namespace protrust {
namespace compiler {

class RustSourceGenerator {
protected:
    RustSourceGenerator(const Options& options);
    virtual ~RustSourceGenerator();

    RustSourceGenerator(const RustSourceGenerator&) = delete;
    RustSourceGenerator& operator=(const RustSourceGenerator&) = delete;

    const Options& options();

private:
    const Options& options_;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_SOURCE_GENERATOR_H__