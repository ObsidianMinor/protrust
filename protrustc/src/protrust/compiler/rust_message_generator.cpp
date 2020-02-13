#include <protrust/compiler/rust_message_generator.h>
#include <protrust/compiler/rust_field_generator.h>
#include <protrust/compiler/rust_helpers.h>

#include <map>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

RustMessageGenerator::RustMessageGenerator(const Descriptor* message, const Options& options)
    : RustSourceGenerator(options), _message(message) { }

RustMessageGenerator::~RustMessageGenerator() { }

void RustMessageGenerator::Generate(io::Printer& printer) {
    const Descriptor* message = this->_message;
    std::map<std::string, std::string> vars;
    vars["name"] = message->name();

    printer.Print(vars,
        "#[derive(Clone, Debug, PartialEq, Default)]\n"
        "pub struct $name$ {\n"
    );
    printer.Indent();

    for (int i = 0; i < message->field_count(); i++) {
        const FieldDescriptor* field = message->field(i);
        RustFieldGenerator generator(field, this->options());
        generator.GenerateStructField(printer);
    }

    if (message->extension_range_count() != 0) {
        printer.Print("__extensions: __prelude::ExtensionSet<Self>,\n");
    }

    printer.Print("__unknown_fields: __prelude::UnknownFieldSet,\n");

    printer.Outdent();
    printer.Print(vars,
        "}\n"
        "impl __prelude::Message for self::$name$ {\n"
    );
    printer.Indent();

    printer.Print(
        "fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {\n"
    );
    printer.Indent();

    printer.Print(
        "while let __prelude::Some(field) = input.read_field() {\n"
    );
    printer.Indent();

    printer.Print(
        "match field.tag() {\n"
    );
    printer.Indent();

    for (int i = 0; i < message->field_count(); i++) {
        const FieldDescriptor* field = message->field(i);
        RustFieldGenerator generator(field, this->options());
        generator.GenerateMergeBranches(printer);
    }

    if (message->extension_range_count() != 0) {
        printer.Print(
            "_ => \n"
            "  field\n"
            "    .check_and_try_add_field_to(&mut self.__extensions)?\n"
            "    .or_try(&mut self.__unknown_fields)?\n"
            "    .or_skip()?\n"
        );
    }
    else {
        printer.Print(
            "_ => \n"
            "  field\n"
            "    .check_and_try_add_field_to(&mut self.__unknown_fields)?\n"
            "    .or_skip()?\n"
        );
    }

    printer.Outdent(); // match
    printer.Print(
        "}\n"
    );
    printer.Outdent(); // while
    printer.Print(
        "}\n"
    );
    printer.Outdent(); // fn merge_from
    printer.Print(
        "}\n"
        "fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {\n"
    );
    printer.Indent();

    // unimplemented

    printer.Outdent(); // fn calculate_size
    printer.Print(
        "}\n"
        "fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::Codedwriter<T>) -> __prelude::write::Result {\n"
    );
    printer.Indent();

    // unimplemented

    printer.Outdent(); // fn write_to
    printer.Print(
        "}\n"
        "fn is_initialized(&self) -> bool {\n"
    );
    printer.Indent();

    // unimplemented

    printer.Outdent(); // fn is_initialized
    printer.Print(
        "}\n"
        "fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {\n"
        "  &self.__unknown_fields\n"
        "}\n"
        "fn unknown_fields_mut(&mut self) -> &mut __prelude::UnkownFieldSet {\n"
        "  &mut self.__unknown_fields\n"
        "}\n"
    );

    printer.Outdent(); // impl Message
    printer.Print(vars,
        "}\n"
        "prefl::dbg_msg!(self::$name$ { full_name: \"$full_name$\", name: \"$name$\" });\n"
    );

    const Descriptor* containing_type = message->containing_type();
    if (containing_type != NULL) {
        printer.Print(vars,
            "prefl::msg_type!(self::$name$: &<super::$parent$ as __prelude::MessageType>::descriptor().nested_type()[$index$]);\n"
        );
    }
    else {
        printer.Print(vars,
            "prefl::msg_type!(self::$name$: &super::file().message_type()[$index$]);\n"
        );
    }

    printer.Print(
        "impl self::$name$ {\n"
    );
    printer.Indent();

    for (int i = 0; i < message->field_count(); i++) {
        const FieldDescriptor* field = message->field(i);
        RustFieldGenerator generator(field, this->options());
        generator.GenerateItems(printer);
    }

    printer.Outdent();
    printer.Print(
        "}\n"
    );

    if (HasInnerItems(message)) {
        printer.Print(
            "pub mod $mod_name$ {\n"
        );
        printer.Indent();

        for (int i = 0; i < message->nested_type_count(); i++) {
            const Descriptor* nested_type = message->nested_type(i);
        }

        printer.Outdent();
        printer.Print(
            "}\n"
        );
    }
}

} // compiler
} // protrust