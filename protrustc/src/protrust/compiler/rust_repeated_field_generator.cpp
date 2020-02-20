#include <protrust/compiler/rust_repeated_field_generator.h>
#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_helpers.h>
#include <protrust/compiler/rust_names.h>

#include <stdexcept>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustRepeatedFieldGenerator::RustRepeatedFieldGenerator(const FieldDescriptor* field, const Options& options)
    : RustFieldGenerator(field, options) { }

RustRepeatedFieldGenerator::~RustRepeatedFieldGenerator() { }

void RustRepeatedFieldGenerator::GenerateMergeBranches(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = field->name();
    vars["arg"] = this->GetImplGenericArg();
    vars["num"] = GetFieldNumberName(field);

    uint tag = MakeTag(field->number(), GetWireType(field->type()));
    vars["unpacked"] = std::to_string(tag);

    if (field->is_packable()) {
        uint packed_tag = MakeTag(field->number(), WIRE_TYPE::WT_LENGTH_DELIMITED);
        vars["packed"] = std::to_string(packed_tag);

        if (field->is_packed()) {
            printer.Print(vars,
                "$packed$ => field.add_entries_to::<_, __prelude::pr::Packed<$arg$>>(Self::$num$, &mut self.$name$)?,\n"
                "$unpacked$ => field.add_entries_to::<_, $arg$>(Self::$num$, &mut self.$name$)?,\n"
            );
        }
        else {
            printer.Print(vars,
                "$unpacked$ => field.add_entries_to::<_, $arg$>(Self::$num$, &mut self.$name$)?,\n"
                "$packed$ => field.add_entries_to::<_, __prelude::pr::Packed<$arg$>>(Self::$num$, &mut self.$name$)?,\n"
            );
        }
    }
    else {
        printer.Print(vars,
            "$unpacked$ => field.add_entries_to::<_, $arg$>(Self::$num$, &mut self.$name$)?,\n"
        );
    }
}

void RustRepeatedFieldGenerator::GenerateCalculateSize(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = field->name();
    vars["arg"] = this->GetImplGenericArg();
    vars["num"] = GetFieldNumberName(field);

    if (field->is_packed()) {
        printer.Print(vars,
            "builder = builder.add_values::<_, __prelude::pr::Packed<$arg$>>(Self::$num$, &self.$name$)?;\n"
        );
    }
    else {
        printer.Print(vars,
            "builder = builder.add_values::<_, $arg$>(Self::$num$, &self.$name$)?;\n"
        );
    }
}

void RustRepeatedFieldGenerator::GenerateWriteTo(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = field->name();
    vars["arg"] = this->GetImplGenericArg();
    vars["num"] = GetFieldNumberName(field);

    if (field->is_packed()) {
        printer.Print(vars,
            "output.write_values::<_, __prelude::pr::Packed<$arg$>>(Self::$num$, &self.$name$)?;\n"
        );
    }
    else {
        printer.Print(vars,
            "output.write_values::<_, $arg$>(Self::$num$, &self.$name$)?;\n"
        );
    }
}

void RustRepeatedFieldGenerator::GenerateIsInitialized(io::Printer& printer) {
    printer.Print(
        "if !__prelude::p::is_initialized(&self.$name$) {\n"
        "  return false;\n"
        "}\n",
        "name", this->field()->name()
    );
}

void RustRepeatedFieldGenerator::GenerateItems(io::Printer& printer) {
    printer.Print(
        "pub fn $name$(&self) -> &$type$ {\n"
        "  &self.$name$\n"
        "}\n"
        "pub fn $name$_mut(&mut self) -> &mut $type$ {\n"
        "  &mut self.$name$\n"
        "}\n",
        "name", GetFieldName(this->field()),
        "type", this->GetFieldType()
    );
}

void RustRepeatedFieldGenerator::GenerateExtension(io::Printer& printer) {
    /*
    if (this->field()->is_packed()) {
        printer.Print(
            "pub static $ext$: __prelude::RepeatedExtension<$containing_type$, __prelude::pr::Packed<$arg$>> = unsafe { __prelude::RepeatedExtension::new_unchecked($num_val$) };\n"
        );
    }
    else {
        printer.Print(
            "pub static $ext$: __prelude::RepeatedExtension<$containing_type$, $arg$> = unsafe { __prelude::RepeatedExtension::new_unchecked($num_val$) };\n"
        );
    }
    */
}

std::string RustRepeatedFieldGenerator::GetFieldType() const {
    return "__prelude::RepeatedField<" + GetRustType(this->field()) + ">";
}

std::string RustRepeatedFieldGenerator::GetImplGenericArg() const {
    return GetRawFieldType(this->field());
}

} // compiler
} // protrust