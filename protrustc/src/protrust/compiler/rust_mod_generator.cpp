#include <protrust/compiler/rust_mod_generator.h>
#include <protrust/compiler/rust_file_generator.h>
#include <protrust/compiler/rust_helpers.h>

#include <vector>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/compiler/code_generator.h>
#include <google/protobuf/io/printer.h>
#include <google/protobuf/io/zero_copy_stream.h>

using namespace google::protobuf;
using namespace google::protobuf::compiler;

namespace protrust {
namespace compiler {

class RustFileGenerator;

RustModGenerator::RustModGenerator(const Options& options) : RustSourceGenerator(options) { }

RustModGenerator::~RustModGenerator() { }

void RustModGenerator::Generate(const std::vector<const FileDescriptor*>& files, GeneratorContext* context) {
    io::ZeroCopyOutputStream* mod_stream = context->Open("mod.rs");
    io::Printer mod_printer(mod_stream, '$');

    mod_printer.PrintRaw("// DO NOT EDIT! This file was generated by protoc-gen-rust as part of the protrust library\n\n");

    for (std::size_t i = 0; i < files.size(); i++) {
        const FileDescriptor* file = files[i];
        this->GenerateFileMod(file, mod_printer);

        std::string file_path = GetOutputFilePath(file, "protrust");
        io::ZeroCopyOutputStream* file_stream = context->Open(file_path);
        io::Printer file_printer(file_stream, '$');

        RustFileGenerator file_generator(file, this->options());
        file_generator.Generate(file_printer);
    }
}

void RustModGenerator::GenerateFileMod(const FileDescriptor* file, io::Printer& printer) {
    std::string file_mod = GetFileModName(file);
    printer.Print(
        "#[path = \"$file_dir$\"]\n"
        "pub mod $file_mod$ {\n",
        "file_dir", GetFileDirPath(file),
        "file_mod", file_mod
    );
    printer.Indent();
    printer.Print(
        "pub(self) use super::globals as __globals;\n"
        // re-use the current module as file, allowing code-gen to continue re-using that ident at any point in the module
        "pub(self) use super::$file_mod$ as __file;\n"
        // re-use the pool and extension registry so plugins can refer to their accessors through file::pool and file::registry
        "file_mod", file_mod
    );

    printer.Print("pub(self) mod __imports {\n");
    printer.Indent();

    for (int i = 0; i < file->dependency_count(); i++) {
        const FileDescriptor* dependency = file->dependency(i);
        printer.Print("pub(super) use super::super::$import$;\n", "import", GetFileModName(dependency));
    }

    printer.Outdent();
    printer.Print("}\n\n");

    printer.Print(
        "#[path = \"protrust.rs\"]\n"
        "mod protrust;\n"
        "\n"
        "pub use self::protrust::*;\n"
        "\n"
    );

    const std::vector<std::string>& imports = this->options().imports;
    for (std::size_t i = 0; i < imports.size(); i++) {
        const std::string& import = imports[i];
        printer.Print(
            "\n"
            "#[path = \"$import$.rs\"]\n"
            "mod $import$;\n"
            "\n"
            "pub use self::$import$::*;\n",
            "import", import
        );
    }

    printer.Outdent();
    printer.Print("}\n");
}

} // compiler
} // protrust