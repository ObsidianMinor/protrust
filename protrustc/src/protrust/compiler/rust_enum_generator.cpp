#include <protrust/compiler/rust_enum_generator.h>
#include <protrust/compiler/rust_source_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_names.h>

#include <string>
#include <map>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/io/printer.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustEnumGenerator::RustEnumGenerator(const EnumDescriptor* enum_type, const Options& options)
    : RustSourceGenerator(options), _enum_type(enum_type) { }

RustEnumGenerator::~RustEnumGenerator() { }

void RustEnumGenerator::Generate(io::Printer& printer) {
    const EnumDescriptor* enum_type = this->_enum_type;
    std::map<std::string, std::string> vars;
    vars["name"] = GetEnumName(enum_type);

    printer.Print(vars,
        "#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n"
        "pub struct $name$(pub i32);\n"
        "\n"
        "impl __prelude::Enum for $name$ { }\n"
        "impl __prelude::From<i32> for $name$ {\n"
        "  fn from(x: i32) -> Self {\n"
        "    Self(x)\n"
        "  }\n"
        "}\n"
        "impl __prelude::From<$name$> for i32 {\n"
        "  fn from(x: $name$) -> Self {\n"
        "    x.0\n"
        "  }\n"
        "}\n"
        "impl __prelude::Default for $name$ {\n"
        "  fn default() -> Self {\n"
        "    Self(0)\n"
        "  }\n"
        "}\n"
    );

    printer.Print(vars,
        "impl $name$ {\n"
    );
    printer.Indent();

    for (int i = 0; i < enum_type->value_count(); i++) {
        const EnumValueDescriptor* value = enum_type->value(i);
        printer.Print(
            "pub const $name$: Self = Self($value$);\n",
            "name", GetEnumValueName(value),
            "value", std::to_string(value->number()));
    }

    printer.Outdent();
    printer.Print(
        "}\n"
    );

    printer.Print(vars,
        "impl __prelude::Debug for $name$ {\n"
    );
    printer.Indent();
    printer.Print(
        "fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {\n"
    );
    printer.Indent();
    printer.Print(
        "#[allow(unreachable_patterns)]\n" // simplifies generated code for now
        "match *self {\n"
    );
    printer.Indent();

    for (int i = 0; i < enum_type->value_count(); i++) {
        const EnumValueDescriptor* value = enum_type->value(i);
        printer.Print(
            "Self::$name$ => f.write_str(\"$name$\"),\n", 
            "name", GetEnumValueName(value)
        );
    }

    printer.Print("Self(x) => x.fmt(f),\n");

    printer.Outdent(); // match
    printer.Print(
        "}\n"
    );
    printer.Outdent(); // fmt
    printer.Print(
        "}\n"
    );
    printer.Outdent(); // impl
    printer.Print(
        "}\n"
    );
}

} // compiler
} // protrust