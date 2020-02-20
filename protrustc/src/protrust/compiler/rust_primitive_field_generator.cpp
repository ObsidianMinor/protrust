#include <protrust/compiler/rust_primitive_field_generator.h>
#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_options.h>
#include <protrust/compiler/rust_helpers.h>
#include <protrust/compiler/rust_names.h>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustPrimitiveFieldGenerator::RustPrimitiveFieldGenerator(const FieldDescriptor* field, const Options& options)
    : RustFieldGenerator(field, options) { }

RustPrimitiveFieldGenerator::~RustPrimitiveFieldGenerator() { }

void RustPrimitiveFieldGenerator::GenerateMergeBranches(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = GetFieldName(field);
    vars["type"] = GetRawFieldType(field);
    vars["num"] = GetFieldNumberName(field);

    uint tag = MakeTag(field->number(), GetWireType(field->type()));
    vars["tag"] = std::to_string(tag);

    if (IsProto2File(field->file())) {
        printer.Print(vars,
            "$tag$ => field.merge_value::<$type$>(Self::$num$, self.$name$.get_or_insert_with(__prelude::Default::default))?,\n"
        );
    }
    else {
        printer.Print(vars,
            "$tag$ => field.merge_value::<$type$>(Self::$num$, &mut self.$name$)?,\n"
        );
    }
}
void RustPrimitiveFieldGenerator::GenerateCalculateSize(io::Printer& printer) {

}
void RustPrimitiveFieldGenerator::GenerateWriteTo(io::Printer& printer) {

}
void RustPrimitiveFieldGenerator::GenerateIsInitialized(io::Printer& printer) {

}
void RustPrimitiveFieldGenerator::GenerateItems(io::Printer& printer) {
    const FieldDescriptor* field = this->field();
    std::map<std::string, std::string> vars;
    vars["name"] = GetFieldName(field);
    vars["name_noescp"] = field->name();
    vars["type"] = GetRustType(field);
    vars["default"] = GetFieldDefaultName(field);
    vars["default_type"] = GetDefaultType(field);
    vars["default_ref"] = GetDefaultTypeRef(field);
    vars["default_val"] = GetDefaultValue(field);

    if (IsProto2File(field->file())) {
        if (IsRustCopyable(field)) {
            printer.Print(vars,
                "pub const $default$: $default_type$ = $default_val$;\n"
                "pub fn $name$(&self) -> $default_ref$ {\n"
                "  self.$name$.unwrap_or(Self::$default$)\n"
                "}\n"
                "pub fn $name_noescp$_option(&self) -> __prelude::Option<&$type$> {\n"
                "  self.$name$.as_ref()\n"
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
                "  self.$name$.take()\n"
                "}\n"
                "pub fn clear_$name_noescp$(&mut self) {\n"
                "  self.$name$ = __prelude::None\n"
                "}\n"
            );
        } else {
            printer.Print(vars,
                "pub const $default$: $default_type$ = $default_val$;\n"
                "pub fn $name$(&self) -> $default_ref$ {\n"
                "  self.$name$.as_ref().map_or(Self::$default$, __prelude::AsRef::as_ref)\n"
                "}\n"
                "pub fn $name_noescp$_option(&self) -> __prelude::Option<&$type$> {\n"
                "  self.$name$.as_ref()\n"
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
                "  self.$name$.take()\n"
                "}\n"
                "pub fn clear_$name_noescp$(&mut self) {\n"
                "  self.$name$ = __prelude::None\n"
                "}\n"
            );
        }
    }
    else {
        printer.Print(vars,
            "pub static $default$: $default_type$ = $default_val$;\n"
            "pub fn $name$(&self) -> &$type$ {\n"
            "  &self.$name$\n"
            "}\n"
            "pub fn $name$_mut(&mut self) -> &mut $type$ {\n"
            "  &mut self.$name$\n"
            "}\n"
        );
    }
}
void RustPrimitiveFieldGenerator::GenerateExtension(io::Printer& printer) {

}

std::string RustPrimitiveFieldGenerator::GetFieldType() const {
    if (IsProto2File(this->field()->file())) {
        return "__prelude::Option<" + GetRustType(this->field()) + ">";
    }
    else {
        return GetRustType(this->field());
    }
}

} // compiler
} // protrust