#include <protrust/compiler/rust_helpers.h>
#include <protrust/compiler/rust_names.h>

#include <string>

#include <google/protobuf/descriptor.h>

using namespace google::protobuf;

namespace protrust {
namespace compiler {

std::string GetMessageModName(const Descriptor* descriptor) {
    const std::string& input = descriptor->name();
    bool last_capped = false;
    std::string result;
    for (std::size_t i = 0; i < input.size(); i++) {
        if ('A' <= input[i] && input[i] <= 'Z') {
            if (i != 0 && !last_capped) {
                result += '_';
            }
            result += input[i] + ('a' - 'A');
        }
        else {
            result += input[i];
        }
    }
    return result;
}

std::string GetEnumValueName(const EnumValueDescriptor* descriptor) {
    return Escape(descriptor->name()); // we can strip prefixes later
}

std::string GetOutputFilePath(const FileDescriptor* file, const char* import_name) {
    std::string result = GetFileDirPath(file);
    result += '/';
    result += import_name;
    result += ".rs";
    return result;
}

std::string GetFileDirPath(const FileDescriptor* file) {
    return file->name();
}

std::string GetFileModName(const FileDescriptor* file) {
    std::string result;
    const std::string& input = file->name();
    for (auto c : input) {
        if (('A' <= c && c <= 'Z') || ('a' <= c && c <= 'z')) {
            result += c;
        }
        else {
            result += '_';
        }
    }
    return result;
}

std::string GetRawFieldType(const FieldDescriptor* field) {
    switch (field->type()) {
        case FieldDescriptor::TYPE_BOOL:
            return "__prelude::pr::Bool";
        case FieldDescriptor::TYPE_BYTES:
            return "__prelude::pr::Bytes<" + GetRustType(field) + ">";
        case FieldDescriptor::TYPE_DOUBLE:
            return "__prelude::pr::Double";
        case FieldDescriptor::TYPE_ENUM:
            return "__prelude::pr::Enum<" + GetRustType(field) + ">";
        case FieldDescriptor::TYPE_FIXED32:
            return "__prelude::pr::Fixed32";
        case FieldDescriptor::TYPE_FIXED64:
            return "__prelude::pr::Fixed64";
        case FieldDescriptor::TYPE_FLOAT:
            return "__prelude::pr::Float";
        case FieldDescriptor::TYPE_GROUP:
            return "__prelude::pr::Group<" + GetRustType(field) + ">";
        case FieldDescriptor::TYPE_INT32:
            return "__prelude::pr::Int32";
        case FieldDescriptor::TYPE_INT64:
            return "__prelude::pr::Int64";
        case FieldDescriptor::TYPE_MESSAGE:
            return "__prelude::pr::Message<" + GetRustType(field) + ">";
        case FieldDescriptor::TYPE_SFIXED32:
            return "__prelude::pr::Sfixed32";
        case FieldDescriptor::TYPE_SFIXED64:
            return "__prelude::pr::Sfixed64";
        case FieldDescriptor::TYPE_SINT32:
            return "__prelude::pr::Sint32";
        case FieldDescriptor::TYPE_SINT64:
            return "__prelude::pr::Sint64";
        case FieldDescriptor::TYPE_STRING:
            return "__prelude::pr::String";
        case FieldDescriptor::TYPE_UINT32:
            return "__prelude::pr::Uint32";
        case FieldDescriptor::TYPE_UINT64:
            return "__prelude::pr::Uint64";
        default:
            return "";
    };
}

std::string GetRustType(const FieldDescriptor* field) {
    switch (field->type()) {
        case FieldDescriptor::TYPE_BOOL:
            return "__prelude::bool";
        case FieldDescriptor::TYPE_BYTES:
            return "__prelude::ByteVec";
        case FieldDescriptor::TYPE_DOUBLE:
            return "__prelude::f64";
        case FieldDescriptor::TYPE_ENUM: {
            const EnumDescriptor* enum_type = field->enum_type();
            std::string result = "__file::";
            if (field->file() != enum_type->file()) {
                result += "__imports::" + GetFileModName(enum_type->file()) + "::";
            }

            const Descriptor* containing_type = enum_type->containing_type();
            std::vector<const Descriptor*> parents;
            while (containing_type != NULL) {
                parents.push_back(containing_type);
                containing_type = containing_type->containing_type();
            }

            for (auto iter = parents.rbegin(); iter != parents.rend(); iter++) {
                const Descriptor* parent = *iter;
                result += GetMessageModName(parent) + "::";
            }

            result += GetEnumName(enum_type);

            return result;
        }
        case FieldDescriptor::TYPE_FIXED32:
        case FieldDescriptor::TYPE_UINT32:
            return "__prelude::u32";
        case FieldDescriptor::TYPE_FIXED64:
        case FieldDescriptor::TYPE_UINT64:
            return "__prelude::u64";
        case FieldDescriptor::TYPE_FLOAT:
            return "__prelude::f32";
        case FieldDescriptor::TYPE_GROUP:
        case FieldDescriptor::TYPE_MESSAGE: {
            const Descriptor* message_type = field->message_type();
            std::string result = "__file::";
            if (field->file() != message_type->file()) {
                result += "__imports::" + GetFileModName(message_type->file()) + "::";
            }

            const Descriptor* containing_type = message_type->containing_type();
            std::vector<const Descriptor*> parents;
            while (containing_type != NULL) {
                parents.push_back(containing_type);
                containing_type = containing_type->containing_type();
            }

            for (auto iter = parents.rbegin(); iter != parents.rend(); iter++) {
                const Descriptor* parent = *iter;
                result += GetMessageModName(parent) + "::";
            }

            result += GetMessageName(message_type);

            return result;
        }
        case FieldDescriptor::TYPE_INT32:
        case FieldDescriptor::TYPE_SFIXED32:
        case FieldDescriptor::TYPE_SINT32:
            return "__prelude::i32";
        case FieldDescriptor::TYPE_INT64:
        case FieldDescriptor::TYPE_SFIXED64:
        case FieldDescriptor::TYPE_SINT64:
            return "__prelude::i64";
        case FieldDescriptor::TYPE_STRING:
            return "__prelude::String";
        default:
            return "";
    };
}

std::string GetDefaultType(const FieldDescriptor* field) {
    switch (field->type()) {
        case FieldDescriptor::TYPE_BYTES:
            return "&'static [__prelude::u8]";
        case FieldDescriptor::TYPE_STRING:
            return "&'static __prelude::str";
        default:
            return GetRustType(field);
    }
}

std::string GetDefaultTypeRef(const FieldDescriptor* field) {
    switch (field->type()) {
        case FieldDescriptor::TYPE_BYTES:
            return "&[__prelude::u8]";
        case FieldDescriptor::TYPE_STRING:
            return "&__prelude::str";
        default:
            return GetRustType(field);
    }
}

std::string GetDefaultValue(const FieldDescriptor* field) {
    switch (field->type()) {
        case FieldDescriptor::TYPE_BOOL:
            return field->default_value_bool() ? "true" : "false";
        case FieldDescriptor::TYPE_BYTES:
            return "b\"" + field->default_value_string() + "\"";
        case FieldDescriptor::TYPE_DOUBLE:
            return std::to_string(field->default_value_double());
        case FieldDescriptor::TYPE_ENUM: {
            const EnumValueDescriptor* enum_value = field->default_value_enum();
            const EnumDescriptor* enum_type = enum_value->type();
            std::string result = "__file::";
            if (field->file() != enum_type->file()) {
                result += "__imports::" + GetFileModName(enum_type->file()) + "::";
            }

            const Descriptor* containing_type = enum_type->containing_type();
            std::vector<const Descriptor*> parents;
            while (containing_type != NULL) {
                parents.push_back(containing_type);
                containing_type = containing_type->containing_type();
            }

            for (auto iter = parents.rbegin(); iter != parents.rend(); iter++) {
                const Descriptor* parent = *iter;
                result += GetMessageModName(parent) + "::";
            }

            result += GetEnumName(enum_type) + "::" + GetEnumValueName(enum_value);

            return result;
        }
        case FieldDescriptor::TYPE_FIXED32:
        case FieldDescriptor::TYPE_UINT32:
            return std::to_string(field->default_value_uint32());
        case FieldDescriptor::TYPE_FIXED64:
        case FieldDescriptor::TYPE_UINT64:
            return std::to_string(field->default_value_uint64());
        case FieldDescriptor::TYPE_FLOAT:
            return std::to_string(field->default_value_float());
        case FieldDescriptor::TYPE_INT32:
        case FieldDescriptor::TYPE_SFIXED32:
        case FieldDescriptor::TYPE_SINT32:
            return std::to_string(field->default_value_int32());
        case FieldDescriptor::TYPE_INT64:
        case FieldDescriptor::TYPE_SFIXED64:
        case FieldDescriptor::TYPE_SINT64:
            return std::to_string(field->default_value_int64());
        case FieldDescriptor::TYPE_STRING:
            return "\"" + field->default_value_string() + "\"";
        default:
            return "";
    }
}

std::string Escape(std::string str) {
    static std::set<std::string> words {
        "as", "break", "const", "continue", "else", "enum", "false", "fn", 
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", 
        "pub", "ref", "return", "static", "struct", "trait", "true", "type", "unsafe", 
        "use", "where", "while", "dyn", "abstract", "become", "box", "do", "final", 
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "async", 
        "await", "try"
    };

    if (words.count(str) != 0) {
        return "r#" + str;
    }
    else {
        return str;
    }
}

std::string GetFieldNumberName(const FieldDescriptor* descriptor) {
    std::string result;
    const std::string& input = descriptor->name();
    for (auto c : input) {
        result += toupper(c);
    }
    return result + "_NUMBER";
}

std::string GetFieldDefaultName(const FieldDescriptor* descriptor) {
    std::string result;
    const std::string& input = descriptor->name();
    for (auto c : input) {
        result += toupper(c);
    }
    return result + "_DEFAULT";
}

} // compiler
} // protrust