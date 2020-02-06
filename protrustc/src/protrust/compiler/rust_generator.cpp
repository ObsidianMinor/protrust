#include <protrust/compiler/rust_generator.h>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/compiler/code_generator.h>

using namespace google::protobuf;
using namespace google::protobuf::compiler;

namespace protrust {
namespace compiler {

bool RustGenerator::Generate(
	const FileDescriptor* file,
	const std::string& parameter,
	GeneratorContext* generator_context,
	std::string* error) const {
	*error = "unimplemented";
	return false;
}

bool RustGenerator::GenerateAll(
	const std::vector<const FileDescriptor*>& files,
	const std::string& parameter,
	GeneratorContext* generator_context,
	std::string* error) const {
	*error = "unimplemented";
	return false;
}

} // compiler
} // protrust