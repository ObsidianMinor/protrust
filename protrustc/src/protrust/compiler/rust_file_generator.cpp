#include <protrust/compiler/rust_file_generator.h>
#include <protrust/compiler/rust_message_generator.h>
#include <protrust/compiler/rust_enum_generator.h>
#include <protrust/compiler/rust_field_generator.h>

#include <memory>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustFileGenerator::RustFileGenerator(const FileDescriptor* file, const Options& options)
    : RustSourceGenerator(options), _file(file) { }

RustFileGenerator::~RustFileGenerator() { }

void RustFileGenerator::Generate(io::Printer& printer) {
    const FileDescriptor* file = this->_file;

    // printer.Print("prefl::file!(file: __globals::pool => \"$name$\");\n\n", "name", file->name());

    printer.Print(
        "pub(self) use super::__file;\n"
        "pub(self) use ::protrust::gen_prelude as __prelude;\n"
        "\n"
    );

    for (int i = 0; i < file->message_type_count(); i++) {
        const Descriptor* message_type = file->message_type(i);
        RustMessageGenerator generator(message_type, this->options());
        generator.Generate(printer);
    }

    for (int i = 0; i < file->enum_type_count(); i++) {
        const EnumDescriptor* enum_type = file->enum_type(i);
        RustEnumGenerator generator(enum_type, this->options());
        generator.Generate(printer);
    }

    for (int i = 0; i < file->extension_count(); i++) {
        const FieldDescriptor* field = file->extension(i);
        std::unique_ptr<RustFieldGenerator> generator = CreateFieldGenerator(field, this->options());
        generator->GenerateExtension(printer);
    }
}

} // compiler
} // protrust