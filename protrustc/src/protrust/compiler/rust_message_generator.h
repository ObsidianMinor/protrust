#ifndef PROTRUSTC_RUST_MESSAGE_GENERATOR_H__
#define PROTRUSTC_RUST_MESSAGE_GENERATOR_H__

#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_source_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustMessageGenerator : public RustSourceGenerator {
public:
    RustMessageGenerator(const google::protobuf::Descriptor* message, const Options& options);
    ~RustMessageGenerator();

    RustMessageGenerator(const RustMessageGenerator&) = delete;
    RustMessageGenerator& operator=(const RustMessageGenerator&) = delete;

    void Generate(google::protobuf::io::Printer& printer);

private:
    const google::protobuf::Descriptor* _message;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_MESSAGE_GENERATOR_H__