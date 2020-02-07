#ifndef PROTRUSTC_RUST_MOD_GENERATOR_H__
#define PROTRUSTC_RUST_MOD_GENERATOR_H__

#include <protrust/compiler/rust_source_generator.h>

#include <vector>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/compiler/code_generator.h>
#include <google/protobuf/io/printer.h>

namespace protrust {
namespace compiler {

class RustModGenerator : public RustSourceGenerator {
public:
    RustModGenerator(const Options& options);
    ~RustModGenerator();

    RustModGenerator(const RustModGenerator&) = delete;
    RustModGenerator& operator=(const RustModGenerator&) = delete;

    void Generate(const std::vector<const google::protobuf::FileDescriptor*>& files, google::protobuf::compiler::GeneratorContext* context);

private:
	void GenerateFileMod(const google::protobuf::FileDescriptor* file, google::protobuf::io::Printer& printer);
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_MOD_GENERATOR_H__