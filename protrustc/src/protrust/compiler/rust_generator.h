#ifndef PROTRUSTC_RUST_GENERATOR_H__
#define PROTRUSTC_RUST_GENERATOR_H__

#include <string>
#include <vector>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/compiler/code_generator.h>

namespace protrust {
namespace compiler {

class RustGenerator : public google::protobuf::compiler::CodeGenerator {
public:
	virtual bool Generate(
		const google::protobuf::FileDescriptor* file,
		const std::string& parameter,
		google::protobuf::compiler::GeneratorContext* generator_context,
		std::string* error) const;
	virtual bool GenerateAll(
		const std::vector<const google::protobuf::FileDescriptor*>& files,
		const std::string& parameter,
		google::protobuf::compiler::GeneratorContext* generator_context,
		std::string* error) const;
};

} // compiler
} // protrust

#endif // PROTRUSTC_RUST_GENERATOR_H__