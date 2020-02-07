#include <protrust/compiler/rust_generator.h>
#include <protrust/compiler/rust_mod_generator.h>
#include <protrust/compiler/rust_options.h>

#include <cstddef>
#include <string>
#include <sstream>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/compiler/code_generator.h>

using namespace google::protobuf;
using namespace google::protobuf::compiler;

namespace protrust {
namespace compiler {

struct Options;

class RustModGenerator;

bool RustGenerator::Generate(
	const FileDescriptor* file,
	const std::string& parameter,
	GeneratorContext* generator_context,
	std::string* error) const {
	*error = "unimplemented; use GenerateAll";
	return false;
}

bool RustGenerator::GenerateAll(
	const std::vector<const FileDescriptor*>& files,
	const std::string& parameter,
	GeneratorContext* generator_context,
	std::string* error) const {
	std::vector<std::pair<std::string, std::string>> options;
	ParseGeneratorParameter(parameter, &options);

	struct Options cli_options;

	for (std::size_t i = 0; i < options.size(); i++) {
		if (options[i].first == "file_extension") {
			cli_options.file_extension = options[i].second;
		} else if (options[i].first == "imports") {
			std::string buf;
			std::stringstream ss(options[i].second);

			while (getline(ss, buf, ',')) {
				cli_options.imports.push_back(buf);
			}
		} else {
			*error = "Unknown generator option: " + options[i].first;
			return false;
		}
	}

	RustModGenerator mod_generator(cli_options);
	mod_generator.Generate(files, generator_context);

	return true;
}

} // compiler
} // protrust