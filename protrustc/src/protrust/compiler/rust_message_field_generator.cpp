#include <protrust/compiler/rust_message_field_generator.h>
#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_helpers.h>
#include <protrust/compiler/rust_names.h>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustMessageFieldGenerator::RustMessageFieldGenerator(const FieldDescriptor* field, const Options& options)
    : RustFieldGenerator(field, options) { }

RustMessageFieldGenerator::~RustMessageFieldGenerator() { }

void RustMessageFieldGenerator::GenerateMergeBranches(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = field->name();
    vars["type"] = GetRawFieldType(field);
    vars["num"] = GetFieldNumberName(field);

    uint tag = MakeTag(field->number(), GetWireType(field->type()));
    vars["tag"] = std::to_string(tag);

    printer.Print(vars,
        "$tag$ =>\n"
        "  match &mut self.$name$ {\n"
        "    __prelude::Some(v) => field.merge_value::<$type$>(Self::$num$, v)?,\n"
        "    opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<$type$>(Self::$num$)?)),\n"
        "  },\n"
    );
}
void RustMessageFieldGenerator::GenerateCalculateSize(io::Printer& printer) {

}
void RustMessageFieldGenerator::GenerateWriteTo(io::Printer& printer) {

}
void RustMessageFieldGenerator::GenerateIsInitialized(io::Printer& printer) {

}
void RustMessageFieldGenerator::GenerateItems(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = GetFieldName(field);
    vars["name_noescp"] = field->name();
    vars["type"] = GetRustType(field);

    printer.Print(vars,
        "pub fn $name_noescp$_option(&self) -> __prelude::Option<&$type$> {\n"
        "  self.$name$.as_deref()\n"
        "}\n"
        "pub fn $name_noescp$_mut(&mut self) -> &mut $type$ {\n"
        "  self.$name$.get_or_insert_with(__prelude::Default::default)\n"
        "}\n"
        "pub fn has_$name_noescp$(&self) -> bool {\n"
        "  self.$name$.is_some()\n"
        "}\n"
        "pub fn set_$name_noescp$(&mut self, value: $type$) {\n"
        "  self.$name$ = __prelude::Some(__prelude::From::from(value))\n"
        "}\n"
        "pub fn take_$name_noescp$(&mut self) -> __prelude::Option<$type$> {\n"
        "  self.$name$.take().map(|v| *v)\n"
        "}\n"
        "pub fn clear_$name_noescp$(&mut self) {\n"
        "  self.$name$ = __prelude::None\n"
        "}\n"
    );
}
void RustMessageFieldGenerator::GenerateExtension(io::Printer& printer) {
    
}

std::string RustMessageFieldGenerator::GetFieldType() const {
    return "__prelude::Option<__prelude::Box<" + GetRustType(this->field()) + ">>";
}

} // compiler
} // protrust