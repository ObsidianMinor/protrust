pub(self) use super::__file;
pub(self) use ::protrust::gen_prelude as __prelude;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDescriptorSet {
  file: __prelude::RepeatedField<__file::FileDescriptorProto>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::FileDescriptorSet {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.add_entries_to::<_, __prelude::pr::Message<__file::FileDescriptorProto>>(Self::FILE_NUMBER, &mut self.file)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::FileDescriptorProto>>(Self::FILE_NUMBER, &self.file)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::FileDescriptorProto>>(Self::FILE_NUMBER, &self.file)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::FileDescriptorSet {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.file) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::FileDescriptorSet { full_name: "google.protobuf.FileDescriptorSet", name: "FileDescriptorSet" });
impl self::FileDescriptorSet {
  pub const FILE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub fn file(&self) -> &__prelude::RepeatedField<__file::FileDescriptorProto> {
    &self.file
  }
  pub fn file_mut(&mut self) -> &mut __prelude::RepeatedField<__file::FileDescriptorProto> {
    &mut self.file
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  package: __prelude::Option<__prelude::String>,
  dependency: __prelude::RepeatedField<__prelude::String>,
  public_dependency: __prelude::RepeatedField<__prelude::i32>,
  weak_dependency: __prelude::RepeatedField<__prelude::i32>,
  message_type: __prelude::RepeatedField<__file::DescriptorProto>,
  enum_type: __prelude::RepeatedField<__file::EnumDescriptorProto>,
  service: __prelude::RepeatedField<__file::ServiceDescriptorProto>,
  extension: __prelude::RepeatedField<__file::FieldDescriptorProto>,
  options: __prelude::Option<__prelude::Box<__file::FileOptions>>,
  source_code_info: __prelude::Option<__prelude::Box<__file::SourceCodeInfo>>,
  syntax: __prelude::Option<__prelude::String>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::FileDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.merge_value::<__prelude::pr::String>(Self::PACKAGE_NUMBER, self.package.get_or_insert_with(__prelude::Default::default))?,
        26 => field.add_entries_to::<_, __prelude::pr::String>(Self::DEPENDENCY_NUMBER, &mut self.dependency)?,
        80 => field.add_entries_to::<_, __prelude::pr::Int32>(Self::PUBLIC_DEPENDENCY_NUMBER, &mut self.public_dependency)?,
        82 => field.add_entries_to::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PUBLIC_DEPENDENCY_NUMBER, &mut self.public_dependency)?,
        88 => field.add_entries_to::<_, __prelude::pr::Int32>(Self::WEAK_DEPENDENCY_NUMBER, &mut self.weak_dependency)?,
        90 => field.add_entries_to::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::WEAK_DEPENDENCY_NUMBER, &mut self.weak_dependency)?,
        34 => field.add_entries_to::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::MESSAGE_TYPE_NUMBER, &mut self.message_type)?,
        42 => field.add_entries_to::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &mut self.enum_type)?,
        50 => field.add_entries_to::<_, __prelude::pr::Message<__file::ServiceDescriptorProto>>(Self::SERVICE_NUMBER, &mut self.service)?,
        58 => field.add_entries_to::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &mut self.extension)?,
        66 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::FileOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::FileOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        74 =>
          match &mut self.source_code_info {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::SourceCodeInfo>>(Self::SOURCE_CODE_INFO_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::SourceCodeInfo>>(Self::SOURCE_CODE_INFO_NUMBER)?)),
          },
        98 => field.merge_value::<__prelude::pr::String>(Self::SYNTAX_NUMBER, self.syntax.get_or_insert_with(__prelude::Default::default))?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::String>(Self::DEPENDENCY_NUMBER, &self.dependency)?;
    builder = builder.add_values::<_, __prelude::pr::Int32>(Self::PUBLIC_DEPENDENCY_NUMBER, &self.public_dependency)?;
    builder = builder.add_values::<_, __prelude::pr::Int32>(Self::WEAK_DEPENDENCY_NUMBER, &self.weak_dependency)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::MESSAGE_TYPE_NUMBER, &self.message_type)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &self.enum_type)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::ServiceDescriptorProto>>(Self::SERVICE_NUMBER, &self.service)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &self.extension)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::String>(Self::DEPENDENCY_NUMBER, &self.dependency)?;
    output.write_values::<_, __prelude::pr::Int32>(Self::PUBLIC_DEPENDENCY_NUMBER, &self.public_dependency)?;
    output.write_values::<_, __prelude::pr::Int32>(Self::WEAK_DEPENDENCY_NUMBER, &self.weak_dependency)?;
    output.write_values::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::MESSAGE_TYPE_NUMBER, &self.message_type)?;
    output.write_values::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &self.enum_type)?;
    output.write_values::<_, __prelude::pr::Message<__file::ServiceDescriptorProto>>(Self::SERVICE_NUMBER, &self.service)?;
    output.write_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &self.extension)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::FileDescriptorProto {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.dependency) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.public_dependency) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.weak_dependency) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.message_type) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.enum_type) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.service) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.extension) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::FileDescriptorProto { full_name: "google.protobuf.FileDescriptorProto", name: "FileDescriptorProto" });
impl self::FileDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const PACKAGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const PACKAGE_DEFAULT: &'static __prelude::str = "";
  pub fn package(&self) -> &__prelude::str {
    self.package.as_ref().map_or(Self::PACKAGE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn package_option(&self) -> __prelude::Option<&__prelude::String> {
    self.package.as_ref()
  }
  pub fn package_mut(&mut self) -> &mut __prelude::String {
    self.package.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_package(&self) -> bool {
    self.package.is_some()
  }
  pub fn set_package(&mut self, value: __prelude::String) {
    self.package = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_package(&mut self) -> __prelude::Option<__prelude::String> {
    self.package.take()
  }
  pub fn clear_package(&mut self) {
    self.package = __prelude::None
  }
  pub const DEPENDENCY_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub fn dependency(&self) -> &__prelude::RepeatedField<__prelude::String> {
    &self.dependency
  }
  pub fn dependency_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::String> {
    &mut self.dependency
  }
  pub const PUBLIC_DEPENDENCY_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(10) };
  pub fn public_dependency(&self) -> &__prelude::RepeatedField<__prelude::i32> {
    &self.public_dependency
  }
  pub fn public_dependency_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::i32> {
    &mut self.public_dependency
  }
  pub const WEAK_DEPENDENCY_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(11) };
  pub fn weak_dependency(&self) -> &__prelude::RepeatedField<__prelude::i32> {
    &self.weak_dependency
  }
  pub fn weak_dependency_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::i32> {
    &mut self.weak_dependency
  }
  pub const MESSAGE_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub fn message_type(&self) -> &__prelude::RepeatedField<__file::DescriptorProto> {
    &self.message_type
  }
  pub fn message_type_mut(&mut self) -> &mut __prelude::RepeatedField<__file::DescriptorProto> {
    &mut self.message_type
  }
  pub const ENUM_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub fn enum_type(&self) -> &__prelude::RepeatedField<__file::EnumDescriptorProto> {
    &self.enum_type
  }
  pub fn enum_type_mut(&mut self) -> &mut __prelude::RepeatedField<__file::EnumDescriptorProto> {
    &mut self.enum_type
  }
  pub const SERVICE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub fn service(&self) -> &__prelude::RepeatedField<__file::ServiceDescriptorProto> {
    &self.service
  }
  pub fn service_mut(&mut self) -> &mut __prelude::RepeatedField<__file::ServiceDescriptorProto> {
    &mut self.service
  }
  pub const EXTENSION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(7) };
  pub fn extension(&self) -> &__prelude::RepeatedField<__file::FieldDescriptorProto> {
    &self.extension
  }
  pub fn extension_mut(&mut self) -> &mut __prelude::RepeatedField<__file::FieldDescriptorProto> {
    &mut self.extension
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(8) };
  pub fn options_option(&self) -> __prelude::Option<&__file::FileOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::FileOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::FileOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::FileOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
  pub const SOURCE_CODE_INFO_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(9) };
  pub fn source_code_info_option(&self) -> __prelude::Option<&__file::SourceCodeInfo> {
    self.source_code_info.as_deref()
  }
  pub fn source_code_info_mut(&mut self) -> &mut __file::SourceCodeInfo {
    self.source_code_info.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_source_code_info(&self) -> bool {
    self.source_code_info.is_some()
  }
  pub fn set_source_code_info(&mut self, value: __file::SourceCodeInfo) {
    self.source_code_info = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_source_code_info(&mut self) -> __prelude::Option<__file::SourceCodeInfo> {
    self.source_code_info.take().map(|v| *v)
  }
  pub fn clear_source_code_info(&mut self) {
    self.source_code_info = __prelude::None
  }
  pub const SYNTAX_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(12) };
  pub const SYNTAX_DEFAULT: &'static __prelude::str = "";
  pub fn syntax(&self) -> &__prelude::str {
    self.syntax.as_ref().map_or(Self::SYNTAX_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn syntax_option(&self) -> __prelude::Option<&__prelude::String> {
    self.syntax.as_ref()
  }
  pub fn syntax_mut(&mut self) -> &mut __prelude::String {
    self.syntax.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_syntax(&self) -> bool {
    self.syntax.is_some()
  }
  pub fn set_syntax(&mut self, value: __prelude::String) {
    self.syntax = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_syntax(&mut self) -> __prelude::Option<__prelude::String> {
    self.syntax.take()
  }
  pub fn clear_syntax(&mut self) {
    self.syntax = __prelude::None
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DescriptorProto {
  name: __prelude::Option<__prelude::String>,
  field: __prelude::RepeatedField<__file::FieldDescriptorProto>,
  extension: __prelude::RepeatedField<__file::FieldDescriptorProto>,
  nested_type: __prelude::RepeatedField<__file::DescriptorProto>,
  enum_type: __prelude::RepeatedField<__file::EnumDescriptorProto>,
  extension_range: __prelude::RepeatedField<__file::descriptor_proto::ExtensionRange>,
  oneof_decl: __prelude::RepeatedField<__file::OneofDescriptorProto>,
  options: __prelude::Option<__prelude::Box<__file::MessageOptions>>,
  reserved_range: __prelude::RepeatedField<__file::descriptor_proto::ReservedRange>,
  reserved_name: __prelude::RepeatedField<__prelude::String>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::DescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.add_entries_to::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::FIELD_NUMBER, &mut self.field)?,
        50 => field.add_entries_to::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &mut self.extension)?,
        26 => field.add_entries_to::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::NESTED_TYPE_NUMBER, &mut self.nested_type)?,
        34 => field.add_entries_to::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &mut self.enum_type)?,
        42 => field.add_entries_to::<_, __prelude::pr::Message<__file::descriptor_proto::ExtensionRange>>(Self::EXTENSION_RANGE_NUMBER, &mut self.extension_range)?,
        66 => field.add_entries_to::<_, __prelude::pr::Message<__file::OneofDescriptorProto>>(Self::ONEOF_DECL_NUMBER, &mut self.oneof_decl)?,
        58 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::MessageOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::MessageOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        74 => field.add_entries_to::<_, __prelude::pr::Message<__file::descriptor_proto::ReservedRange>>(Self::RESERVED_RANGE_NUMBER, &mut self.reserved_range)?,
        82 => field.add_entries_to::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &mut self.reserved_name)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::FIELD_NUMBER, &self.field)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &self.extension)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::NESTED_TYPE_NUMBER, &self.nested_type)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &self.enum_type)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::descriptor_proto::ExtensionRange>>(Self::EXTENSION_RANGE_NUMBER, &self.extension_range)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::OneofDescriptorProto>>(Self::ONEOF_DECL_NUMBER, &self.oneof_decl)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::descriptor_proto::ReservedRange>>(Self::RESERVED_RANGE_NUMBER, &self.reserved_range)?;
    builder = builder.add_values::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &self.reserved_name)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::FIELD_NUMBER, &self.field)?;
    output.write_values::<_, __prelude::pr::Message<__file::FieldDescriptorProto>>(Self::EXTENSION_NUMBER, &self.extension)?;
    output.write_values::<_, __prelude::pr::Message<__file::DescriptorProto>>(Self::NESTED_TYPE_NUMBER, &self.nested_type)?;
    output.write_values::<_, __prelude::pr::Message<__file::EnumDescriptorProto>>(Self::ENUM_TYPE_NUMBER, &self.enum_type)?;
    output.write_values::<_, __prelude::pr::Message<__file::descriptor_proto::ExtensionRange>>(Self::EXTENSION_RANGE_NUMBER, &self.extension_range)?;
    output.write_values::<_, __prelude::pr::Message<__file::OneofDescriptorProto>>(Self::ONEOF_DECL_NUMBER, &self.oneof_decl)?;
    output.write_values::<_, __prelude::pr::Message<__file::descriptor_proto::ReservedRange>>(Self::RESERVED_RANGE_NUMBER, &self.reserved_range)?;
    output.write_values::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &self.reserved_name)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::DescriptorProto {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.field) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.extension) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.nested_type) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.enum_type) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.extension_range) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.oneof_decl) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.reserved_range) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.reserved_name) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::DescriptorProto { full_name: "google.protobuf.DescriptorProto", name: "DescriptorProto" });
impl self::DescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const FIELD_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub fn field(&self) -> &__prelude::RepeatedField<__file::FieldDescriptorProto> {
    &self.field
  }
  pub fn field_mut(&mut self) -> &mut __prelude::RepeatedField<__file::FieldDescriptorProto> {
    &mut self.field
  }
  pub const EXTENSION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub fn extension(&self) -> &__prelude::RepeatedField<__file::FieldDescriptorProto> {
    &self.extension
  }
  pub fn extension_mut(&mut self) -> &mut __prelude::RepeatedField<__file::FieldDescriptorProto> {
    &mut self.extension
  }
  pub const NESTED_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub fn nested_type(&self) -> &__prelude::RepeatedField<__file::DescriptorProto> {
    &self.nested_type
  }
  pub fn nested_type_mut(&mut self) -> &mut __prelude::RepeatedField<__file::DescriptorProto> {
    &mut self.nested_type
  }
  pub const ENUM_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub fn enum_type(&self) -> &__prelude::RepeatedField<__file::EnumDescriptorProto> {
    &self.enum_type
  }
  pub fn enum_type_mut(&mut self) -> &mut __prelude::RepeatedField<__file::EnumDescriptorProto> {
    &mut self.enum_type
  }
  pub const EXTENSION_RANGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub fn extension_range(&self) -> &__prelude::RepeatedField<__file::descriptor_proto::ExtensionRange> {
    &self.extension_range
  }
  pub fn extension_range_mut(&mut self) -> &mut __prelude::RepeatedField<__file::descriptor_proto::ExtensionRange> {
    &mut self.extension_range
  }
  pub const ONEOF_DECL_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(8) };
  pub fn oneof_decl(&self) -> &__prelude::RepeatedField<__file::OneofDescriptorProto> {
    &self.oneof_decl
  }
  pub fn oneof_decl_mut(&mut self) -> &mut __prelude::RepeatedField<__file::OneofDescriptorProto> {
    &mut self.oneof_decl
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(7) };
  pub fn options_option(&self) -> __prelude::Option<&__file::MessageOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::MessageOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::MessageOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::MessageOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
  pub const RESERVED_RANGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(9) };
  pub fn reserved_range(&self) -> &__prelude::RepeatedField<__file::descriptor_proto::ReservedRange> {
    &self.reserved_range
  }
  pub fn reserved_range_mut(&mut self) -> &mut __prelude::RepeatedField<__file::descriptor_proto::ReservedRange> {
    &mut self.reserved_range
  }
  pub const RESERVED_NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(10) };
  pub fn reserved_name(&self) -> &__prelude::RepeatedField<__prelude::String> {
    &self.reserved_name
  }
  pub fn reserved_name_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::String> {
    &mut self.reserved_name
  }
}
pub mod descriptor_proto {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct ExtensionRange {
    start: __prelude::Option<__prelude::i32>,
    end: __prelude::Option<__prelude::i32>,
    options: __prelude::Option<__prelude::Box<__file::ExtensionRangeOptions>>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::ExtensionRange {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          8 => field.merge_value::<__prelude::pr::Int32>(Self::START_NUMBER, self.start.get_or_insert_with(__prelude::Default::default))?,
          16 => field.merge_value::<__prelude::pr::Int32>(Self::END_NUMBER, self.end.get_or_insert_with(__prelude::Default::default))?,
          26 =>
            match &mut self.options {
              __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::ExtensionRangeOptions>>(Self::OPTIONS_NUMBER, v)?,
              opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::ExtensionRangeOptions>>(Self::OPTIONS_NUMBER)?)),
            },
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::ExtensionRange {
    fn is_initialized(&self) -> bool {
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::ExtensionRange { full_name: "google.protobuf.DescriptorProto.ExtensionRange", name: "ExtensionRange" });
  impl self::ExtensionRange {
    pub const START_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub const START_DEFAULT: __prelude::i32 = 0;
    pub fn start(&self) -> __prelude::i32 {
      self.start.unwrap_or(Self::START_DEFAULT)
    }
    pub fn start_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.start.as_ref()
    }
    pub fn start_mut(&mut self) -> &mut __prelude::i32 {
      self.start.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_start(&self) -> bool {
      self.start.is_some()
    }
    pub fn set_start(&mut self, value: __prelude::i32) {
      self.start = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_start(&mut self) -> __prelude::Option<__prelude::i32> {
      self.start.take()
    }
    pub fn clear_start(&mut self) {
      self.start = __prelude::None
    }
    pub const END_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub const END_DEFAULT: __prelude::i32 = 0;
    pub fn end(&self) -> __prelude::i32 {
      self.end.unwrap_or(Self::END_DEFAULT)
    }
    pub fn end_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.end.as_ref()
    }
    pub fn end_mut(&mut self) -> &mut __prelude::i32 {
      self.end.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_end(&self) -> bool {
      self.end.is_some()
    }
    pub fn set_end(&mut self, value: __prelude::i32) {
      self.end = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_end(&mut self) -> __prelude::Option<__prelude::i32> {
      self.end.take()
    }
    pub fn clear_end(&mut self) {
      self.end = __prelude::None
    }
    pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
    pub fn options_option(&self) -> __prelude::Option<&__file::ExtensionRangeOptions> {
      self.options.as_deref()
    }
    pub fn options_mut(&mut self) -> &mut __file::ExtensionRangeOptions {
      self.options.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_options(&self) -> bool {
      self.options.is_some()
    }
    pub fn set_options(&mut self, value: __file::ExtensionRangeOptions) {
      self.options = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_options(&mut self) -> __prelude::Option<__file::ExtensionRangeOptions> {
      self.options.take().map(|v| *v)
    }
    pub fn clear_options(&mut self) {
      self.options = __prelude::None
    }
  }
  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct ReservedRange {
    start: __prelude::Option<__prelude::i32>,
    end: __prelude::Option<__prelude::i32>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::ReservedRange {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          8 => field.merge_value::<__prelude::pr::Int32>(Self::START_NUMBER, self.start.get_or_insert_with(__prelude::Default::default))?,
          16 => field.merge_value::<__prelude::pr::Int32>(Self::END_NUMBER, self.end.get_or_insert_with(__prelude::Default::default))?,
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::ReservedRange {
    fn is_initialized(&self) -> bool {
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::ReservedRange { full_name: "google.protobuf.DescriptorProto.ReservedRange", name: "ReservedRange" });
  impl self::ReservedRange {
    pub const START_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub const START_DEFAULT: __prelude::i32 = 0;
    pub fn start(&self) -> __prelude::i32 {
      self.start.unwrap_or(Self::START_DEFAULT)
    }
    pub fn start_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.start.as_ref()
    }
    pub fn start_mut(&mut self) -> &mut __prelude::i32 {
      self.start.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_start(&self) -> bool {
      self.start.is_some()
    }
    pub fn set_start(&mut self, value: __prelude::i32) {
      self.start = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_start(&mut self) -> __prelude::Option<__prelude::i32> {
      self.start.take()
    }
    pub fn clear_start(&mut self) {
      self.start = __prelude::None
    }
    pub const END_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub const END_DEFAULT: __prelude::i32 = 0;
    pub fn end(&self) -> __prelude::i32 {
      self.end.unwrap_or(Self::END_DEFAULT)
    }
    pub fn end_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.end.as_ref()
    }
    pub fn end_mut(&mut self) -> &mut __prelude::i32 {
      self.end.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_end(&self) -> bool {
      self.end.is_some()
    }
    pub fn set_end(&mut self, value: __prelude::i32) {
      self.end = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_end(&mut self) -> __prelude::Option<__prelude::i32> {
      self.end.take()
    }
    pub fn clear_end(&mut self) {
      self.end = __prelude::None
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ExtensionRangeOptions {
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::ExtensionRangeOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::ExtensionRangeOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::ExtensionRangeOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::ExtensionRangeOptions { full_name: "google.protobuf.ExtensionRangeOptions", name: "ExtensionRangeOptions" });
impl self::ExtensionRangeOptions {
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FieldDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  number: __prelude::Option<__prelude::i32>,
  label: __prelude::Option<__file::field_descriptor_proto::Label>,
  r#type: __prelude::Option<__file::field_descriptor_proto::Type>,
  type_name: __prelude::Option<__prelude::String>,
  extendee: __prelude::Option<__prelude::String>,
  default_value: __prelude::Option<__prelude::String>,
  oneof_index: __prelude::Option<__prelude::i32>,
  json_name: __prelude::Option<__prelude::String>,
  options: __prelude::Option<__prelude::Box<__file::FieldOptions>>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::FieldDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        24 => field.merge_value::<__prelude::pr::Int32>(Self::NUMBER_NUMBER, self.number.get_or_insert_with(__prelude::Default::default))?,
        32 => field.merge_value::<__prelude::pr::Enum<__file::field_descriptor_proto::Label>>(Self::LABEL_NUMBER, self.label.get_or_insert_with(__prelude::Default::default))?,
        40 => field.merge_value::<__prelude::pr::Enum<__file::field_descriptor_proto::Type>>(Self::TYPE_NUMBER, self.r#type.get_or_insert_with(__prelude::Default::default))?,
        50 => field.merge_value::<__prelude::pr::String>(Self::TYPE_NAME_NUMBER, self.type_name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.merge_value::<__prelude::pr::String>(Self::EXTENDEE_NUMBER, self.extendee.get_or_insert_with(__prelude::Default::default))?,
        58 => field.merge_value::<__prelude::pr::String>(Self::DEFAULT_VALUE_NUMBER, self.default_value.get_or_insert_with(__prelude::Default::default))?,
        72 => field.merge_value::<__prelude::pr::Int32>(Self::ONEOF_INDEX_NUMBER, self.oneof_index.get_or_insert_with(__prelude::Default::default))?,
        82 => field.merge_value::<__prelude::pr::String>(Self::JSON_NAME_NUMBER, self.json_name.get_or_insert_with(__prelude::Default::default))?,
        66 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::FieldOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::FieldOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::FieldDescriptorProto {
  fn is_initialized(&self) -> bool {
    true
  }
}
__prelude::prefl::dbg_msg!(self::FieldDescriptorProto { full_name: "google.protobuf.FieldDescriptorProto", name: "FieldDescriptorProto" });
impl self::FieldDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const NUMBER_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const NUMBER_DEFAULT: __prelude::i32 = 0;
  pub fn number(&self) -> __prelude::i32 {
    self.number.unwrap_or(Self::NUMBER_DEFAULT)
  }
  pub fn number_option(&self) -> __prelude::Option<&__prelude::i32> {
    self.number.as_ref()
  }
  pub fn number_mut(&mut self) -> &mut __prelude::i32 {
    self.number.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_number(&self) -> bool {
    self.number.is_some()
  }
  pub fn set_number(&mut self, value: __prelude::i32) {
    self.number = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_number(&mut self) -> __prelude::Option<__prelude::i32> {
    self.number.take()
  }
  pub fn clear_number(&mut self) {
    self.number = __prelude::None
  }
  pub const LABEL_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub const LABEL_DEFAULT: __file::field_descriptor_proto::Label = __file::field_descriptor_proto::Label::LABEL_OPTIONAL;
  pub fn label(&self) -> __file::field_descriptor_proto::Label {
    self.label.unwrap_or(Self::LABEL_DEFAULT)
  }
  pub fn label_option(&self) -> __prelude::Option<&__file::field_descriptor_proto::Label> {
    self.label.as_ref()
  }
  pub fn label_mut(&mut self) -> &mut __file::field_descriptor_proto::Label {
    self.label.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_label(&self) -> bool {
    self.label.is_some()
  }
  pub fn set_label(&mut self, value: __file::field_descriptor_proto::Label) {
    self.label = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_label(&mut self) -> __prelude::Option<__file::field_descriptor_proto::Label> {
    self.label.take()
  }
  pub fn clear_label(&mut self) {
    self.label = __prelude::None
  }
  pub const TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub const TYPE_DEFAULT: __file::field_descriptor_proto::Type = __file::field_descriptor_proto::Type::TYPE_DOUBLE;
  pub fn r#type(&self) -> __file::field_descriptor_proto::Type {
    self.r#type.unwrap_or(Self::TYPE_DEFAULT)
  }
  pub fn type_option(&self) -> __prelude::Option<&__file::field_descriptor_proto::Type> {
    self.r#type.as_ref()
  }
  pub fn type_mut(&mut self) -> &mut __file::field_descriptor_proto::Type {
    self.r#type.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_type(&self) -> bool {
    self.r#type.is_some()
  }
  pub fn set_type(&mut self, value: __file::field_descriptor_proto::Type) {
    self.r#type = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_type(&mut self) -> __prelude::Option<__file::field_descriptor_proto::Type> {
    self.r#type.take()
  }
  pub fn clear_type(&mut self) {
    self.r#type = __prelude::None
  }
  pub const TYPE_NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub const TYPE_NAME_DEFAULT: &'static __prelude::str = "";
  pub fn type_name(&self) -> &__prelude::str {
    self.type_name.as_ref().map_or(Self::TYPE_NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn type_name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.type_name.as_ref()
  }
  pub fn type_name_mut(&mut self) -> &mut __prelude::String {
    self.type_name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_type_name(&self) -> bool {
    self.type_name.is_some()
  }
  pub fn set_type_name(&mut self, value: __prelude::String) {
    self.type_name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_type_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.type_name.take()
  }
  pub fn clear_type_name(&mut self) {
    self.type_name = __prelude::None
  }
  pub const EXTENDEE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const EXTENDEE_DEFAULT: &'static __prelude::str = "";
  pub fn extendee(&self) -> &__prelude::str {
    self.extendee.as_ref().map_or(Self::EXTENDEE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn extendee_option(&self) -> __prelude::Option<&__prelude::String> {
    self.extendee.as_ref()
  }
  pub fn extendee_mut(&mut self) -> &mut __prelude::String {
    self.extendee.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_extendee(&self) -> bool {
    self.extendee.is_some()
  }
  pub fn set_extendee(&mut self, value: __prelude::String) {
    self.extendee = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_extendee(&mut self) -> __prelude::Option<__prelude::String> {
    self.extendee.take()
  }
  pub fn clear_extendee(&mut self) {
    self.extendee = __prelude::None
  }
  pub const DEFAULT_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(7) };
  pub const DEFAULT_VALUE_DEFAULT: &'static __prelude::str = "";
  pub fn default_value(&self) -> &__prelude::str {
    self.default_value.as_ref().map_or(Self::DEFAULT_VALUE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn default_value_option(&self) -> __prelude::Option<&__prelude::String> {
    self.default_value.as_ref()
  }
  pub fn default_value_mut(&mut self) -> &mut __prelude::String {
    self.default_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_default_value(&self) -> bool {
    self.default_value.is_some()
  }
  pub fn set_default_value(&mut self, value: __prelude::String) {
    self.default_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_default_value(&mut self) -> __prelude::Option<__prelude::String> {
    self.default_value.take()
  }
  pub fn clear_default_value(&mut self) {
    self.default_value = __prelude::None
  }
  pub const ONEOF_INDEX_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(9) };
  pub const ONEOF_INDEX_DEFAULT: __prelude::i32 = 0;
  pub fn oneof_index(&self) -> __prelude::i32 {
    self.oneof_index.unwrap_or(Self::ONEOF_INDEX_DEFAULT)
  }
  pub fn oneof_index_option(&self) -> __prelude::Option<&__prelude::i32> {
    self.oneof_index.as_ref()
  }
  pub fn oneof_index_mut(&mut self) -> &mut __prelude::i32 {
    self.oneof_index.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_oneof_index(&self) -> bool {
    self.oneof_index.is_some()
  }
  pub fn set_oneof_index(&mut self, value: __prelude::i32) {
    self.oneof_index = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_oneof_index(&mut self) -> __prelude::Option<__prelude::i32> {
    self.oneof_index.take()
  }
  pub fn clear_oneof_index(&mut self) {
    self.oneof_index = __prelude::None
  }
  pub const JSON_NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(10) };
  pub const JSON_NAME_DEFAULT: &'static __prelude::str = "";
  pub fn json_name(&self) -> &__prelude::str {
    self.json_name.as_ref().map_or(Self::JSON_NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn json_name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.json_name.as_ref()
  }
  pub fn json_name_mut(&mut self) -> &mut __prelude::String {
    self.json_name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_json_name(&self) -> bool {
    self.json_name.is_some()
  }
  pub fn set_json_name(&mut self, value: __prelude::String) {
    self.json_name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_json_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.json_name.take()
  }
  pub fn clear_json_name(&mut self) {
    self.json_name = __prelude::None
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(8) };
  pub fn options_option(&self) -> __prelude::Option<&__file::FieldOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::FieldOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::FieldOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::FieldOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
}
pub mod field_descriptor_proto {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct Type(pub i32);

  impl __prelude::Enum for Type { }
  impl __prelude::From<i32> for Type {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<Type> for i32 {
    fn from(x: Type) -> Self {
      x.0
    }
  }
  impl __prelude::Default for Type {
    fn default() -> Self {
      Self(0)
    }
  }
  impl Type {
    pub const TYPE_DOUBLE: Self = Self(1);
    pub const TYPE_FLOAT: Self = Self(2);
    pub const TYPE_INT64: Self = Self(3);
    pub const TYPE_UINT64: Self = Self(4);
    pub const TYPE_INT32: Self = Self(5);
    pub const TYPE_FIXED64: Self = Self(6);
    pub const TYPE_FIXED32: Self = Self(7);
    pub const TYPE_BOOL: Self = Self(8);
    pub const TYPE_STRING: Self = Self(9);
    pub const TYPE_GROUP: Self = Self(10);
    pub const TYPE_MESSAGE: Self = Self(11);
    pub const TYPE_BYTES: Self = Self(12);
    pub const TYPE_UINT32: Self = Self(13);
    pub const TYPE_ENUM: Self = Self(14);
    pub const TYPE_SFIXED32: Self = Self(15);
    pub const TYPE_SFIXED64: Self = Self(16);
    pub const TYPE_SINT32: Self = Self(17);
    pub const TYPE_SINT64: Self = Self(18);
  }
  impl __prelude::Debug for Type {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::TYPE_DOUBLE => f.write_str("TYPE_DOUBLE"),
        Self::TYPE_FLOAT => f.write_str("TYPE_FLOAT"),
        Self::TYPE_INT64 => f.write_str("TYPE_INT64"),
        Self::TYPE_UINT64 => f.write_str("TYPE_UINT64"),
        Self::TYPE_INT32 => f.write_str("TYPE_INT32"),
        Self::TYPE_FIXED64 => f.write_str("TYPE_FIXED64"),
        Self::TYPE_FIXED32 => f.write_str("TYPE_FIXED32"),
        Self::TYPE_BOOL => f.write_str("TYPE_BOOL"),
        Self::TYPE_STRING => f.write_str("TYPE_STRING"),
        Self::TYPE_GROUP => f.write_str("TYPE_GROUP"),
        Self::TYPE_MESSAGE => f.write_str("TYPE_MESSAGE"),
        Self::TYPE_BYTES => f.write_str("TYPE_BYTES"),
        Self::TYPE_UINT32 => f.write_str("TYPE_UINT32"),
        Self::TYPE_ENUM => f.write_str("TYPE_ENUM"),
        Self::TYPE_SFIXED32 => f.write_str("TYPE_SFIXED32"),
        Self::TYPE_SFIXED64 => f.write_str("TYPE_SFIXED64"),
        Self::TYPE_SINT32 => f.write_str("TYPE_SINT32"),
        Self::TYPE_SINT64 => f.write_str("TYPE_SINT64"),
        Self(x) => x.fmt(f),
      }
    }
  }
  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct Label(pub i32);

  impl __prelude::Enum for Label { }
  impl __prelude::From<i32> for Label {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<Label> for i32 {
    fn from(x: Label) -> Self {
      x.0
    }
  }
  impl __prelude::Default for Label {
    fn default() -> Self {
      Self(0)
    }
  }
  impl Label {
    pub const LABEL_OPTIONAL: Self = Self(1);
    pub const LABEL_REQUIRED: Self = Self(2);
    pub const LABEL_REPEATED: Self = Self(3);
  }
  impl __prelude::Debug for Label {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::LABEL_OPTIONAL => f.write_str("LABEL_OPTIONAL"),
        Self::LABEL_REQUIRED => f.write_str("LABEL_REQUIRED"),
        Self::LABEL_REPEATED => f.write_str("LABEL_REPEATED"),
        Self(x) => x.fmt(f),
      }
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct OneofDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  options: __prelude::Option<__prelude::Box<__file::OneofOptions>>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::OneofDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::OneofOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::OneofOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::OneofDescriptorProto {
  fn is_initialized(&self) -> bool {
    true
  }
}
__prelude::prefl::dbg_msg!(self::OneofDescriptorProto { full_name: "google.protobuf.OneofDescriptorProto", name: "OneofDescriptorProto" });
impl self::OneofDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub fn options_option(&self) -> __prelude::Option<&__file::OneofOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::OneofOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::OneofOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::OneofOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnumDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  value: __prelude::RepeatedField<__file::EnumValueDescriptorProto>,
  options: __prelude::Option<__prelude::Box<__file::EnumOptions>>,
  reserved_range: __prelude::RepeatedField<__file::enum_descriptor_proto::EnumReservedRange>,
  reserved_name: __prelude::RepeatedField<__prelude::String>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::EnumDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.add_entries_to::<_, __prelude::pr::Message<__file::EnumValueDescriptorProto>>(Self::VALUE_NUMBER, &mut self.value)?,
        26 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::EnumOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::EnumOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        34 => field.add_entries_to::<_, __prelude::pr::Message<__file::enum_descriptor_proto::EnumReservedRange>>(Self::RESERVED_RANGE_NUMBER, &mut self.reserved_range)?,
        42 => field.add_entries_to::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &mut self.reserved_name)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::EnumValueDescriptorProto>>(Self::VALUE_NUMBER, &self.value)?;
    builder = builder.add_values::<_, __prelude::pr::Message<__file::enum_descriptor_proto::EnumReservedRange>>(Self::RESERVED_RANGE_NUMBER, &self.reserved_range)?;
    builder = builder.add_values::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &self.reserved_name)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::EnumValueDescriptorProto>>(Self::VALUE_NUMBER, &self.value)?;
    output.write_values::<_, __prelude::pr::Message<__file::enum_descriptor_proto::EnumReservedRange>>(Self::RESERVED_RANGE_NUMBER, &self.reserved_range)?;
    output.write_values::<_, __prelude::pr::String>(Self::RESERVED_NAME_NUMBER, &self.reserved_name)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::EnumDescriptorProto {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.value) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.reserved_range) {
      return false;
    }
    if !__prelude::p::is_initialized(&self.reserved_name) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::EnumDescriptorProto { full_name: "google.protobuf.EnumDescriptorProto", name: "EnumDescriptorProto" });
impl self::EnumDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub fn value(&self) -> &__prelude::RepeatedField<__file::EnumValueDescriptorProto> {
    &self.value
  }
  pub fn value_mut(&mut self) -> &mut __prelude::RepeatedField<__file::EnumValueDescriptorProto> {
    &mut self.value
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub fn options_option(&self) -> __prelude::Option<&__file::EnumOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::EnumOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::EnumOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::EnumOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
  pub const RESERVED_RANGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub fn reserved_range(&self) -> &__prelude::RepeatedField<__file::enum_descriptor_proto::EnumReservedRange> {
    &self.reserved_range
  }
  pub fn reserved_range_mut(&mut self) -> &mut __prelude::RepeatedField<__file::enum_descriptor_proto::EnumReservedRange> {
    &mut self.reserved_range
  }
  pub const RESERVED_NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub fn reserved_name(&self) -> &__prelude::RepeatedField<__prelude::String> {
    &self.reserved_name
  }
  pub fn reserved_name_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::String> {
    &mut self.reserved_name
  }
}
pub mod enum_descriptor_proto {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct EnumReservedRange {
    start: __prelude::Option<__prelude::i32>,
    end: __prelude::Option<__prelude::i32>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::EnumReservedRange {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          8 => field.merge_value::<__prelude::pr::Int32>(Self::START_NUMBER, self.start.get_or_insert_with(__prelude::Default::default))?,
          16 => field.merge_value::<__prelude::pr::Int32>(Self::END_NUMBER, self.end.get_or_insert_with(__prelude::Default::default))?,
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::EnumReservedRange {
    fn is_initialized(&self) -> bool {
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::EnumReservedRange { full_name: "google.protobuf.EnumDescriptorProto.EnumReservedRange", name: "EnumReservedRange" });
  impl self::EnumReservedRange {
    pub const START_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub const START_DEFAULT: __prelude::i32 = 0;
    pub fn start(&self) -> __prelude::i32 {
      self.start.unwrap_or(Self::START_DEFAULT)
    }
    pub fn start_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.start.as_ref()
    }
    pub fn start_mut(&mut self) -> &mut __prelude::i32 {
      self.start.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_start(&self) -> bool {
      self.start.is_some()
    }
    pub fn set_start(&mut self, value: __prelude::i32) {
      self.start = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_start(&mut self) -> __prelude::Option<__prelude::i32> {
      self.start.take()
    }
    pub fn clear_start(&mut self) {
      self.start = __prelude::None
    }
    pub const END_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub const END_DEFAULT: __prelude::i32 = 0;
    pub fn end(&self) -> __prelude::i32 {
      self.end.unwrap_or(Self::END_DEFAULT)
    }
    pub fn end_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.end.as_ref()
    }
    pub fn end_mut(&mut self) -> &mut __prelude::i32 {
      self.end.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_end(&self) -> bool {
      self.end.is_some()
    }
    pub fn set_end(&mut self, value: __prelude::i32) {
      self.end = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_end(&mut self) -> __prelude::Option<__prelude::i32> {
      self.end.take()
    }
    pub fn clear_end(&mut self) {
      self.end = __prelude::None
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnumValueDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  number: __prelude::Option<__prelude::i32>,
  options: __prelude::Option<__prelude::Box<__file::EnumValueOptions>>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::EnumValueDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        16 => field.merge_value::<__prelude::pr::Int32>(Self::NUMBER_NUMBER, self.number.get_or_insert_with(__prelude::Default::default))?,
        26 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::EnumValueOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::EnumValueOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::EnumValueDescriptorProto {
  fn is_initialized(&self) -> bool {
    true
  }
}
__prelude::prefl::dbg_msg!(self::EnumValueDescriptorProto { full_name: "google.protobuf.EnumValueDescriptorProto", name: "EnumValueDescriptorProto" });
impl self::EnumValueDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const NUMBER_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const NUMBER_DEFAULT: __prelude::i32 = 0;
  pub fn number(&self) -> __prelude::i32 {
    self.number.unwrap_or(Self::NUMBER_DEFAULT)
  }
  pub fn number_option(&self) -> __prelude::Option<&__prelude::i32> {
    self.number.as_ref()
  }
  pub fn number_mut(&mut self) -> &mut __prelude::i32 {
    self.number.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_number(&self) -> bool {
    self.number.is_some()
  }
  pub fn set_number(&mut self, value: __prelude::i32) {
    self.number = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_number(&mut self) -> __prelude::Option<__prelude::i32> {
    self.number.take()
  }
  pub fn clear_number(&mut self) {
    self.number = __prelude::None
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub fn options_option(&self) -> __prelude::Option<&__file::EnumValueOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::EnumValueOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::EnumValueOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::EnumValueOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ServiceDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  method: __prelude::RepeatedField<__file::MethodDescriptorProto>,
  options: __prelude::Option<__prelude::Box<__file::ServiceOptions>>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::ServiceDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.add_entries_to::<_, __prelude::pr::Message<__file::MethodDescriptorProto>>(Self::METHOD_NUMBER, &mut self.method)?,
        26 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::ServiceOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::ServiceOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::MethodDescriptorProto>>(Self::METHOD_NUMBER, &self.method)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::MethodDescriptorProto>>(Self::METHOD_NUMBER, &self.method)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::ServiceDescriptorProto {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.method) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::ServiceDescriptorProto { full_name: "google.protobuf.ServiceDescriptorProto", name: "ServiceDescriptorProto" });
impl self::ServiceDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const METHOD_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub fn method(&self) -> &__prelude::RepeatedField<__file::MethodDescriptorProto> {
    &self.method
  }
  pub fn method_mut(&mut self) -> &mut __prelude::RepeatedField<__file::MethodDescriptorProto> {
    &mut self.method
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub fn options_option(&self) -> __prelude::Option<&__file::ServiceOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::ServiceOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::ServiceOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::ServiceOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MethodDescriptorProto {
  name: __prelude::Option<__prelude::String>,
  input_type: __prelude::Option<__prelude::String>,
  output_type: __prelude::Option<__prelude::String>,
  options: __prelude::Option<__prelude::Box<__file::MethodOptions>>,
  client_streaming: __prelude::Option<__prelude::bool>,
  server_streaming: __prelude::Option<__prelude::bool>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::MethodDescriptorProto {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::NAME_NUMBER, self.name.get_or_insert_with(__prelude::Default::default))?,
        18 => field.merge_value::<__prelude::pr::String>(Self::INPUT_TYPE_NUMBER, self.input_type.get_or_insert_with(__prelude::Default::default))?,
        26 => field.merge_value::<__prelude::pr::String>(Self::OUTPUT_TYPE_NUMBER, self.output_type.get_or_insert_with(__prelude::Default::default))?,
        34 =>
          match &mut self.options {
            __prelude::Some(v) => field.merge_value::<__prelude::pr::Message<__file::MethodOptions>>(Self::OPTIONS_NUMBER, v)?,
            opt @ __prelude::None => *opt = __prelude::Some(__prelude::Box::new(field.read_value::<__prelude::pr::Message<__file::MethodOptions>>(Self::OPTIONS_NUMBER)?)),
          },
        40 => field.merge_value::<__prelude::pr::Bool>(Self::CLIENT_STREAMING_NUMBER, self.client_streaming.get_or_insert_with(__prelude::Default::default))?,
        48 => field.merge_value::<__prelude::pr::Bool>(Self::SERVER_STREAMING_NUMBER, self.server_streaming.get_or_insert_with(__prelude::Default::default))?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::MethodDescriptorProto {
  fn is_initialized(&self) -> bool {
    true
  }
}
__prelude::prefl::dbg_msg!(self::MethodDescriptorProto { full_name: "google.protobuf.MethodDescriptorProto", name: "MethodDescriptorProto" });
impl self::MethodDescriptorProto {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const NAME_DEFAULT: &'static __prelude::str = "";
  pub fn name(&self) -> &__prelude::str {
    self.name.as_ref().map_or(Self::NAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn name_option(&self) -> __prelude::Option<&__prelude::String> {
    self.name.as_ref()
  }
  pub fn name_mut(&mut self) -> &mut __prelude::String {
    self.name.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_name(&self) -> bool {
    self.name.is_some()
  }
  pub fn set_name(&mut self, value: __prelude::String) {
    self.name = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_name(&mut self) -> __prelude::Option<__prelude::String> {
    self.name.take()
  }
  pub fn clear_name(&mut self) {
    self.name = __prelude::None
  }
  pub const INPUT_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const INPUT_TYPE_DEFAULT: &'static __prelude::str = "";
  pub fn input_type(&self) -> &__prelude::str {
    self.input_type.as_ref().map_or(Self::INPUT_TYPE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn input_type_option(&self) -> __prelude::Option<&__prelude::String> {
    self.input_type.as_ref()
  }
  pub fn input_type_mut(&mut self) -> &mut __prelude::String {
    self.input_type.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_input_type(&self) -> bool {
    self.input_type.is_some()
  }
  pub fn set_input_type(&mut self, value: __prelude::String) {
    self.input_type = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_input_type(&mut self) -> __prelude::Option<__prelude::String> {
    self.input_type.take()
  }
  pub fn clear_input_type(&mut self) {
    self.input_type = __prelude::None
  }
  pub const OUTPUT_TYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const OUTPUT_TYPE_DEFAULT: &'static __prelude::str = "";
  pub fn output_type(&self) -> &__prelude::str {
    self.output_type.as_ref().map_or(Self::OUTPUT_TYPE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn output_type_option(&self) -> __prelude::Option<&__prelude::String> {
    self.output_type.as_ref()
  }
  pub fn output_type_mut(&mut self) -> &mut __prelude::String {
    self.output_type.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_output_type(&self) -> bool {
    self.output_type.is_some()
  }
  pub fn set_output_type(&mut self, value: __prelude::String) {
    self.output_type = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_output_type(&mut self) -> __prelude::Option<__prelude::String> {
    self.output_type.take()
  }
  pub fn clear_output_type(&mut self) {
    self.output_type = __prelude::None
  }
  pub const OPTIONS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub fn options_option(&self) -> __prelude::Option<&__file::MethodOptions> {
    self.options.as_deref()
  }
  pub fn options_mut(&mut self) -> &mut __file::MethodOptions {
    self.options.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_options(&self) -> bool {
    self.options.is_some()
  }
  pub fn set_options(&mut self, value: __file::MethodOptions) {
    self.options = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_options(&mut self) -> __prelude::Option<__file::MethodOptions> {
    self.options.take().map(|v| *v)
  }
  pub fn clear_options(&mut self) {
    self.options = __prelude::None
  }
  pub const CLIENT_STREAMING_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub const CLIENT_STREAMING_DEFAULT: __prelude::bool = false;
  pub fn client_streaming(&self) -> __prelude::bool {
    self.client_streaming.unwrap_or(Self::CLIENT_STREAMING_DEFAULT)
  }
  pub fn client_streaming_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.client_streaming.as_ref()
  }
  pub fn client_streaming_mut(&mut self) -> &mut __prelude::bool {
    self.client_streaming.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_client_streaming(&self) -> bool {
    self.client_streaming.is_some()
  }
  pub fn set_client_streaming(&mut self, value: __prelude::bool) {
    self.client_streaming = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_client_streaming(&mut self) -> __prelude::Option<__prelude::bool> {
    self.client_streaming.take()
  }
  pub fn clear_client_streaming(&mut self) {
    self.client_streaming = __prelude::None
  }
  pub const SERVER_STREAMING_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub const SERVER_STREAMING_DEFAULT: __prelude::bool = false;
  pub fn server_streaming(&self) -> __prelude::bool {
    self.server_streaming.unwrap_or(Self::SERVER_STREAMING_DEFAULT)
  }
  pub fn server_streaming_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.server_streaming.as_ref()
  }
  pub fn server_streaming_mut(&mut self) -> &mut __prelude::bool {
    self.server_streaming.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_server_streaming(&self) -> bool {
    self.server_streaming.is_some()
  }
  pub fn set_server_streaming(&mut self, value: __prelude::bool) {
    self.server_streaming = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_server_streaming(&mut self) -> __prelude::Option<__prelude::bool> {
    self.server_streaming.take()
  }
  pub fn clear_server_streaming(&mut self) {
    self.server_streaming = __prelude::None
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileOptions {
  java_package: __prelude::Option<__prelude::String>,
  java_outer_classname: __prelude::Option<__prelude::String>,
  java_multiple_files: __prelude::Option<__prelude::bool>,
  java_generate_equals_and_hash: __prelude::Option<__prelude::bool>,
  java_string_check_utf8: __prelude::Option<__prelude::bool>,
  optimize_for: __prelude::Option<__file::file_options::OptimizeMode>,
  go_package: __prelude::Option<__prelude::String>,
  cc_generic_services: __prelude::Option<__prelude::bool>,
  java_generic_services: __prelude::Option<__prelude::bool>,
  py_generic_services: __prelude::Option<__prelude::bool>,
  php_generic_services: __prelude::Option<__prelude::bool>,
  deprecated: __prelude::Option<__prelude::bool>,
  cc_enable_arenas: __prelude::Option<__prelude::bool>,
  objc_class_prefix: __prelude::Option<__prelude::String>,
  csharp_namespace: __prelude::Option<__prelude::String>,
  swift_prefix: __prelude::Option<__prelude::String>,
  php_class_prefix: __prelude::Option<__prelude::String>,
  php_namespace: __prelude::Option<__prelude::String>,
  php_metadata_namespace: __prelude::Option<__prelude::String>,
  ruby_package: __prelude::Option<__prelude::String>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::FileOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.merge_value::<__prelude::pr::String>(Self::JAVA_PACKAGE_NUMBER, self.java_package.get_or_insert_with(__prelude::Default::default))?,
        66 => field.merge_value::<__prelude::pr::String>(Self::JAVA_OUTER_CLASSNAME_NUMBER, self.java_outer_classname.get_or_insert_with(__prelude::Default::default))?,
        80 => field.merge_value::<__prelude::pr::Bool>(Self::JAVA_MULTIPLE_FILES_NUMBER, self.java_multiple_files.get_or_insert_with(__prelude::Default::default))?,
        160 => field.merge_value::<__prelude::pr::Bool>(Self::JAVA_GENERATE_EQUALS_AND_HASH_NUMBER, self.java_generate_equals_and_hash.get_or_insert_with(__prelude::Default::default))?,
        216 => field.merge_value::<__prelude::pr::Bool>(Self::JAVA_STRING_CHECK_UTF8_NUMBER, self.java_string_check_utf8.get_or_insert_with(__prelude::Default::default))?,
        72 => field.merge_value::<__prelude::pr::Enum<__file::file_options::OptimizeMode>>(Self::OPTIMIZE_FOR_NUMBER, self.optimize_for.get_or_insert_with(__prelude::Default::default))?,
        90 => field.merge_value::<__prelude::pr::String>(Self::GO_PACKAGE_NUMBER, self.go_package.get_or_insert_with(__prelude::Default::default))?,
        128 => field.merge_value::<__prelude::pr::Bool>(Self::CC_GENERIC_SERVICES_NUMBER, self.cc_generic_services.get_or_insert_with(__prelude::Default::default))?,
        136 => field.merge_value::<__prelude::pr::Bool>(Self::JAVA_GENERIC_SERVICES_NUMBER, self.java_generic_services.get_or_insert_with(__prelude::Default::default))?,
        144 => field.merge_value::<__prelude::pr::Bool>(Self::PY_GENERIC_SERVICES_NUMBER, self.py_generic_services.get_or_insert_with(__prelude::Default::default))?,
        336 => field.merge_value::<__prelude::pr::Bool>(Self::PHP_GENERIC_SERVICES_NUMBER, self.php_generic_services.get_or_insert_with(__prelude::Default::default))?,
        184 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        248 => field.merge_value::<__prelude::pr::Bool>(Self::CC_ENABLE_ARENAS_NUMBER, self.cc_enable_arenas.get_or_insert_with(__prelude::Default::default))?,
        290 => field.merge_value::<__prelude::pr::String>(Self::OBJC_CLASS_PREFIX_NUMBER, self.objc_class_prefix.get_or_insert_with(__prelude::Default::default))?,
        298 => field.merge_value::<__prelude::pr::String>(Self::CSHARP_NAMESPACE_NUMBER, self.csharp_namespace.get_or_insert_with(__prelude::Default::default))?,
        314 => field.merge_value::<__prelude::pr::String>(Self::SWIFT_PREFIX_NUMBER, self.swift_prefix.get_or_insert_with(__prelude::Default::default))?,
        322 => field.merge_value::<__prelude::pr::String>(Self::PHP_CLASS_PREFIX_NUMBER, self.php_class_prefix.get_or_insert_with(__prelude::Default::default))?,
        330 => field.merge_value::<__prelude::pr::String>(Self::PHP_NAMESPACE_NUMBER, self.php_namespace.get_or_insert_with(__prelude::Default::default))?,
        354 => field.merge_value::<__prelude::pr::String>(Self::PHP_METADATA_NAMESPACE_NUMBER, self.php_metadata_namespace.get_or_insert_with(__prelude::Default::default))?,
        362 => field.merge_value::<__prelude::pr::String>(Self::RUBY_PACKAGE_NUMBER, self.ruby_package.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::FileOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::FileOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::FileOptions { full_name: "google.protobuf.FileOptions", name: "FileOptions" });
impl self::FileOptions {
  pub const JAVA_PACKAGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const JAVA_PACKAGE_DEFAULT: &'static __prelude::str = "";
  pub fn java_package(&self) -> &__prelude::str {
    self.java_package.as_ref().map_or(Self::JAVA_PACKAGE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn java_package_option(&self) -> __prelude::Option<&__prelude::String> {
    self.java_package.as_ref()
  }
  pub fn java_package_mut(&mut self) -> &mut __prelude::String {
    self.java_package.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_package(&self) -> bool {
    self.java_package.is_some()
  }
  pub fn set_java_package(&mut self, value: __prelude::String) {
    self.java_package = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_package(&mut self) -> __prelude::Option<__prelude::String> {
    self.java_package.take()
  }
  pub fn clear_java_package(&mut self) {
    self.java_package = __prelude::None
  }
  pub const JAVA_OUTER_CLASSNAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(8) };
  pub const JAVA_OUTER_CLASSNAME_DEFAULT: &'static __prelude::str = "";
  pub fn java_outer_classname(&self) -> &__prelude::str {
    self.java_outer_classname.as_ref().map_or(Self::JAVA_OUTER_CLASSNAME_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn java_outer_classname_option(&self) -> __prelude::Option<&__prelude::String> {
    self.java_outer_classname.as_ref()
  }
  pub fn java_outer_classname_mut(&mut self) -> &mut __prelude::String {
    self.java_outer_classname.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_outer_classname(&self) -> bool {
    self.java_outer_classname.is_some()
  }
  pub fn set_java_outer_classname(&mut self, value: __prelude::String) {
    self.java_outer_classname = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_outer_classname(&mut self) -> __prelude::Option<__prelude::String> {
    self.java_outer_classname.take()
  }
  pub fn clear_java_outer_classname(&mut self) {
    self.java_outer_classname = __prelude::None
  }
  pub const JAVA_MULTIPLE_FILES_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(10) };
  pub const JAVA_MULTIPLE_FILES_DEFAULT: __prelude::bool = false;
  pub fn java_multiple_files(&self) -> __prelude::bool {
    self.java_multiple_files.unwrap_or(Self::JAVA_MULTIPLE_FILES_DEFAULT)
  }
  pub fn java_multiple_files_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.java_multiple_files.as_ref()
  }
  pub fn java_multiple_files_mut(&mut self) -> &mut __prelude::bool {
    self.java_multiple_files.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_multiple_files(&self) -> bool {
    self.java_multiple_files.is_some()
  }
  pub fn set_java_multiple_files(&mut self, value: __prelude::bool) {
    self.java_multiple_files = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_multiple_files(&mut self) -> __prelude::Option<__prelude::bool> {
    self.java_multiple_files.take()
  }
  pub fn clear_java_multiple_files(&mut self) {
    self.java_multiple_files = __prelude::None
  }
  pub const JAVA_GENERATE_EQUALS_AND_HASH_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(20) };
  pub const JAVA_GENERATE_EQUALS_AND_HASH_DEFAULT: __prelude::bool = false;
  pub fn java_generate_equals_and_hash(&self) -> __prelude::bool {
    self.java_generate_equals_and_hash.unwrap_or(Self::JAVA_GENERATE_EQUALS_AND_HASH_DEFAULT)
  }
  pub fn java_generate_equals_and_hash_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.java_generate_equals_and_hash.as_ref()
  }
  pub fn java_generate_equals_and_hash_mut(&mut self) -> &mut __prelude::bool {
    self.java_generate_equals_and_hash.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_generate_equals_and_hash(&self) -> bool {
    self.java_generate_equals_and_hash.is_some()
  }
  pub fn set_java_generate_equals_and_hash(&mut self, value: __prelude::bool) {
    self.java_generate_equals_and_hash = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_generate_equals_and_hash(&mut self) -> __prelude::Option<__prelude::bool> {
    self.java_generate_equals_and_hash.take()
  }
  pub fn clear_java_generate_equals_and_hash(&mut self) {
    self.java_generate_equals_and_hash = __prelude::None
  }
  pub const JAVA_STRING_CHECK_UTF8_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(27) };
  pub const JAVA_STRING_CHECK_UTF8_DEFAULT: __prelude::bool = false;
  pub fn java_string_check_utf8(&self) -> __prelude::bool {
    self.java_string_check_utf8.unwrap_or(Self::JAVA_STRING_CHECK_UTF8_DEFAULT)
  }
  pub fn java_string_check_utf8_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.java_string_check_utf8.as_ref()
  }
  pub fn java_string_check_utf8_mut(&mut self) -> &mut __prelude::bool {
    self.java_string_check_utf8.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_string_check_utf8(&self) -> bool {
    self.java_string_check_utf8.is_some()
  }
  pub fn set_java_string_check_utf8(&mut self, value: __prelude::bool) {
    self.java_string_check_utf8 = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_string_check_utf8(&mut self) -> __prelude::Option<__prelude::bool> {
    self.java_string_check_utf8.take()
  }
  pub fn clear_java_string_check_utf8(&mut self) {
    self.java_string_check_utf8 = __prelude::None
  }
  pub const OPTIMIZE_FOR_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(9) };
  pub const OPTIMIZE_FOR_DEFAULT: __file::file_options::OptimizeMode = __file::file_options::OptimizeMode::SPEED;
  pub fn optimize_for(&self) -> __file::file_options::OptimizeMode {
    self.optimize_for.unwrap_or(Self::OPTIMIZE_FOR_DEFAULT)
  }
  pub fn optimize_for_option(&self) -> __prelude::Option<&__file::file_options::OptimizeMode> {
    self.optimize_for.as_ref()
  }
  pub fn optimize_for_mut(&mut self) -> &mut __file::file_options::OptimizeMode {
    self.optimize_for.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_optimize_for(&self) -> bool {
    self.optimize_for.is_some()
  }
  pub fn set_optimize_for(&mut self, value: __file::file_options::OptimizeMode) {
    self.optimize_for = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_optimize_for(&mut self) -> __prelude::Option<__file::file_options::OptimizeMode> {
    self.optimize_for.take()
  }
  pub fn clear_optimize_for(&mut self) {
    self.optimize_for = __prelude::None
  }
  pub const GO_PACKAGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(11) };
  pub const GO_PACKAGE_DEFAULT: &'static __prelude::str = "";
  pub fn go_package(&self) -> &__prelude::str {
    self.go_package.as_ref().map_or(Self::GO_PACKAGE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn go_package_option(&self) -> __prelude::Option<&__prelude::String> {
    self.go_package.as_ref()
  }
  pub fn go_package_mut(&mut self) -> &mut __prelude::String {
    self.go_package.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_go_package(&self) -> bool {
    self.go_package.is_some()
  }
  pub fn set_go_package(&mut self, value: __prelude::String) {
    self.go_package = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_go_package(&mut self) -> __prelude::Option<__prelude::String> {
    self.go_package.take()
  }
  pub fn clear_go_package(&mut self) {
    self.go_package = __prelude::None
  }
  pub const CC_GENERIC_SERVICES_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(16) };
  pub const CC_GENERIC_SERVICES_DEFAULT: __prelude::bool = false;
  pub fn cc_generic_services(&self) -> __prelude::bool {
    self.cc_generic_services.unwrap_or(Self::CC_GENERIC_SERVICES_DEFAULT)
  }
  pub fn cc_generic_services_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.cc_generic_services.as_ref()
  }
  pub fn cc_generic_services_mut(&mut self) -> &mut __prelude::bool {
    self.cc_generic_services.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_cc_generic_services(&self) -> bool {
    self.cc_generic_services.is_some()
  }
  pub fn set_cc_generic_services(&mut self, value: __prelude::bool) {
    self.cc_generic_services = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_cc_generic_services(&mut self) -> __prelude::Option<__prelude::bool> {
    self.cc_generic_services.take()
  }
  pub fn clear_cc_generic_services(&mut self) {
    self.cc_generic_services = __prelude::None
  }
  pub const JAVA_GENERIC_SERVICES_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(17) };
  pub const JAVA_GENERIC_SERVICES_DEFAULT: __prelude::bool = false;
  pub fn java_generic_services(&self) -> __prelude::bool {
    self.java_generic_services.unwrap_or(Self::JAVA_GENERIC_SERVICES_DEFAULT)
  }
  pub fn java_generic_services_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.java_generic_services.as_ref()
  }
  pub fn java_generic_services_mut(&mut self) -> &mut __prelude::bool {
    self.java_generic_services.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_java_generic_services(&self) -> bool {
    self.java_generic_services.is_some()
  }
  pub fn set_java_generic_services(&mut self, value: __prelude::bool) {
    self.java_generic_services = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_java_generic_services(&mut self) -> __prelude::Option<__prelude::bool> {
    self.java_generic_services.take()
  }
  pub fn clear_java_generic_services(&mut self) {
    self.java_generic_services = __prelude::None
  }
  pub const PY_GENERIC_SERVICES_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(18) };
  pub const PY_GENERIC_SERVICES_DEFAULT: __prelude::bool = false;
  pub fn py_generic_services(&self) -> __prelude::bool {
    self.py_generic_services.unwrap_or(Self::PY_GENERIC_SERVICES_DEFAULT)
  }
  pub fn py_generic_services_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.py_generic_services.as_ref()
  }
  pub fn py_generic_services_mut(&mut self) -> &mut __prelude::bool {
    self.py_generic_services.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_py_generic_services(&self) -> bool {
    self.py_generic_services.is_some()
  }
  pub fn set_py_generic_services(&mut self, value: __prelude::bool) {
    self.py_generic_services = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_py_generic_services(&mut self) -> __prelude::Option<__prelude::bool> {
    self.py_generic_services.take()
  }
  pub fn clear_py_generic_services(&mut self) {
    self.py_generic_services = __prelude::None
  }
  pub const PHP_GENERIC_SERVICES_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(42) };
  pub const PHP_GENERIC_SERVICES_DEFAULT: __prelude::bool = false;
  pub fn php_generic_services(&self) -> __prelude::bool {
    self.php_generic_services.unwrap_or(Self::PHP_GENERIC_SERVICES_DEFAULT)
  }
  pub fn php_generic_services_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.php_generic_services.as_ref()
  }
  pub fn php_generic_services_mut(&mut self) -> &mut __prelude::bool {
    self.php_generic_services.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_php_generic_services(&self) -> bool {
    self.php_generic_services.is_some()
  }
  pub fn set_php_generic_services(&mut self, value: __prelude::bool) {
    self.php_generic_services = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_php_generic_services(&mut self) -> __prelude::Option<__prelude::bool> {
    self.php_generic_services.take()
  }
  pub fn clear_php_generic_services(&mut self) {
    self.php_generic_services = __prelude::None
  }
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(23) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const CC_ENABLE_ARENAS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(31) };
  pub const CC_ENABLE_ARENAS_DEFAULT: __prelude::bool = false;
  pub fn cc_enable_arenas(&self) -> __prelude::bool {
    self.cc_enable_arenas.unwrap_or(Self::CC_ENABLE_ARENAS_DEFAULT)
  }
  pub fn cc_enable_arenas_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.cc_enable_arenas.as_ref()
  }
  pub fn cc_enable_arenas_mut(&mut self) -> &mut __prelude::bool {
    self.cc_enable_arenas.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_cc_enable_arenas(&self) -> bool {
    self.cc_enable_arenas.is_some()
  }
  pub fn set_cc_enable_arenas(&mut self, value: __prelude::bool) {
    self.cc_enable_arenas = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_cc_enable_arenas(&mut self) -> __prelude::Option<__prelude::bool> {
    self.cc_enable_arenas.take()
  }
  pub fn clear_cc_enable_arenas(&mut self) {
    self.cc_enable_arenas = __prelude::None
  }
  pub const OBJC_CLASS_PREFIX_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(36) };
  pub const OBJC_CLASS_PREFIX_DEFAULT: &'static __prelude::str = "";
  pub fn objc_class_prefix(&self) -> &__prelude::str {
    self.objc_class_prefix.as_ref().map_or(Self::OBJC_CLASS_PREFIX_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn objc_class_prefix_option(&self) -> __prelude::Option<&__prelude::String> {
    self.objc_class_prefix.as_ref()
  }
  pub fn objc_class_prefix_mut(&mut self) -> &mut __prelude::String {
    self.objc_class_prefix.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_objc_class_prefix(&self) -> bool {
    self.objc_class_prefix.is_some()
  }
  pub fn set_objc_class_prefix(&mut self, value: __prelude::String) {
    self.objc_class_prefix = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_objc_class_prefix(&mut self) -> __prelude::Option<__prelude::String> {
    self.objc_class_prefix.take()
  }
  pub fn clear_objc_class_prefix(&mut self) {
    self.objc_class_prefix = __prelude::None
  }
  pub const CSHARP_NAMESPACE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(37) };
  pub const CSHARP_NAMESPACE_DEFAULT: &'static __prelude::str = "";
  pub fn csharp_namespace(&self) -> &__prelude::str {
    self.csharp_namespace.as_ref().map_or(Self::CSHARP_NAMESPACE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn csharp_namespace_option(&self) -> __prelude::Option<&__prelude::String> {
    self.csharp_namespace.as_ref()
  }
  pub fn csharp_namespace_mut(&mut self) -> &mut __prelude::String {
    self.csharp_namespace.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_csharp_namespace(&self) -> bool {
    self.csharp_namespace.is_some()
  }
  pub fn set_csharp_namespace(&mut self, value: __prelude::String) {
    self.csharp_namespace = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_csharp_namespace(&mut self) -> __prelude::Option<__prelude::String> {
    self.csharp_namespace.take()
  }
  pub fn clear_csharp_namespace(&mut self) {
    self.csharp_namespace = __prelude::None
  }
  pub const SWIFT_PREFIX_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(39) };
  pub const SWIFT_PREFIX_DEFAULT: &'static __prelude::str = "";
  pub fn swift_prefix(&self) -> &__prelude::str {
    self.swift_prefix.as_ref().map_or(Self::SWIFT_PREFIX_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn swift_prefix_option(&self) -> __prelude::Option<&__prelude::String> {
    self.swift_prefix.as_ref()
  }
  pub fn swift_prefix_mut(&mut self) -> &mut __prelude::String {
    self.swift_prefix.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_swift_prefix(&self) -> bool {
    self.swift_prefix.is_some()
  }
  pub fn set_swift_prefix(&mut self, value: __prelude::String) {
    self.swift_prefix = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_swift_prefix(&mut self) -> __prelude::Option<__prelude::String> {
    self.swift_prefix.take()
  }
  pub fn clear_swift_prefix(&mut self) {
    self.swift_prefix = __prelude::None
  }
  pub const PHP_CLASS_PREFIX_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(40) };
  pub const PHP_CLASS_PREFIX_DEFAULT: &'static __prelude::str = "";
  pub fn php_class_prefix(&self) -> &__prelude::str {
    self.php_class_prefix.as_ref().map_or(Self::PHP_CLASS_PREFIX_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn php_class_prefix_option(&self) -> __prelude::Option<&__prelude::String> {
    self.php_class_prefix.as_ref()
  }
  pub fn php_class_prefix_mut(&mut self) -> &mut __prelude::String {
    self.php_class_prefix.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_php_class_prefix(&self) -> bool {
    self.php_class_prefix.is_some()
  }
  pub fn set_php_class_prefix(&mut self, value: __prelude::String) {
    self.php_class_prefix = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_php_class_prefix(&mut self) -> __prelude::Option<__prelude::String> {
    self.php_class_prefix.take()
  }
  pub fn clear_php_class_prefix(&mut self) {
    self.php_class_prefix = __prelude::None
  }
  pub const PHP_NAMESPACE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(41) };
  pub const PHP_NAMESPACE_DEFAULT: &'static __prelude::str = "";
  pub fn php_namespace(&self) -> &__prelude::str {
    self.php_namespace.as_ref().map_or(Self::PHP_NAMESPACE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn php_namespace_option(&self) -> __prelude::Option<&__prelude::String> {
    self.php_namespace.as_ref()
  }
  pub fn php_namespace_mut(&mut self) -> &mut __prelude::String {
    self.php_namespace.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_php_namespace(&self) -> bool {
    self.php_namespace.is_some()
  }
  pub fn set_php_namespace(&mut self, value: __prelude::String) {
    self.php_namespace = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_php_namespace(&mut self) -> __prelude::Option<__prelude::String> {
    self.php_namespace.take()
  }
  pub fn clear_php_namespace(&mut self) {
    self.php_namespace = __prelude::None
  }
  pub const PHP_METADATA_NAMESPACE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(44) };
  pub const PHP_METADATA_NAMESPACE_DEFAULT: &'static __prelude::str = "";
  pub fn php_metadata_namespace(&self) -> &__prelude::str {
    self.php_metadata_namespace.as_ref().map_or(Self::PHP_METADATA_NAMESPACE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn php_metadata_namespace_option(&self) -> __prelude::Option<&__prelude::String> {
    self.php_metadata_namespace.as_ref()
  }
  pub fn php_metadata_namespace_mut(&mut self) -> &mut __prelude::String {
    self.php_metadata_namespace.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_php_metadata_namespace(&self) -> bool {
    self.php_metadata_namespace.is_some()
  }
  pub fn set_php_metadata_namespace(&mut self, value: __prelude::String) {
    self.php_metadata_namespace = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_php_metadata_namespace(&mut self) -> __prelude::Option<__prelude::String> {
    self.php_metadata_namespace.take()
  }
  pub fn clear_php_metadata_namespace(&mut self) {
    self.php_metadata_namespace = __prelude::None
  }
  pub const RUBY_PACKAGE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(45) };
  pub const RUBY_PACKAGE_DEFAULT: &'static __prelude::str = "";
  pub fn ruby_package(&self) -> &__prelude::str {
    self.ruby_package.as_ref().map_or(Self::RUBY_PACKAGE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn ruby_package_option(&self) -> __prelude::Option<&__prelude::String> {
    self.ruby_package.as_ref()
  }
  pub fn ruby_package_mut(&mut self) -> &mut __prelude::String {
    self.ruby_package.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_ruby_package(&self) -> bool {
    self.ruby_package.is_some()
  }
  pub fn set_ruby_package(&mut self, value: __prelude::String) {
    self.ruby_package = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_ruby_package(&mut self) -> __prelude::Option<__prelude::String> {
    self.ruby_package.take()
  }
  pub fn clear_ruby_package(&mut self) {
    self.ruby_package = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
pub mod file_options {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct OptimizeMode(pub i32);

  impl __prelude::Enum for OptimizeMode { }
  impl __prelude::From<i32> for OptimizeMode {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<OptimizeMode> for i32 {
    fn from(x: OptimizeMode) -> Self {
      x.0
    }
  }
  impl __prelude::Default for OptimizeMode {
    fn default() -> Self {
      Self(0)
    }
  }
  impl OptimizeMode {
    pub const SPEED: Self = Self(1);
    pub const CODE_SIZE: Self = Self(2);
    pub const LITE_RUNTIME: Self = Self(3);
  }
  impl __prelude::Debug for OptimizeMode {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::SPEED => f.write_str("SPEED"),
        Self::CODE_SIZE => f.write_str("CODE_SIZE"),
        Self::LITE_RUNTIME => f.write_str("LITE_RUNTIME"),
        Self(x) => x.fmt(f),
      }
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MessageOptions {
  message_set_wire_format: __prelude::Option<__prelude::bool>,
  no_standard_descriptor_accessor: __prelude::Option<__prelude::bool>,
  deprecated: __prelude::Option<__prelude::bool>,
  map_entry: __prelude::Option<__prelude::bool>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::MessageOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        8 => field.merge_value::<__prelude::pr::Bool>(Self::MESSAGE_SET_WIRE_FORMAT_NUMBER, self.message_set_wire_format.get_or_insert_with(__prelude::Default::default))?,
        16 => field.merge_value::<__prelude::pr::Bool>(Self::NO_STANDARD_DESCRIPTOR_ACCESSOR_NUMBER, self.no_standard_descriptor_accessor.get_or_insert_with(__prelude::Default::default))?,
        24 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        56 => field.merge_value::<__prelude::pr::Bool>(Self::MAP_ENTRY_NUMBER, self.map_entry.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::MessageOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::MessageOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::MessageOptions { full_name: "google.protobuf.MessageOptions", name: "MessageOptions" });
impl self::MessageOptions {
  pub const MESSAGE_SET_WIRE_FORMAT_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const MESSAGE_SET_WIRE_FORMAT_DEFAULT: __prelude::bool = false;
  pub fn message_set_wire_format(&self) -> __prelude::bool {
    self.message_set_wire_format.unwrap_or(Self::MESSAGE_SET_WIRE_FORMAT_DEFAULT)
  }
  pub fn message_set_wire_format_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.message_set_wire_format.as_ref()
  }
  pub fn message_set_wire_format_mut(&mut self) -> &mut __prelude::bool {
    self.message_set_wire_format.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_message_set_wire_format(&self) -> bool {
    self.message_set_wire_format.is_some()
  }
  pub fn set_message_set_wire_format(&mut self, value: __prelude::bool) {
    self.message_set_wire_format = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_message_set_wire_format(&mut self) -> __prelude::Option<__prelude::bool> {
    self.message_set_wire_format.take()
  }
  pub fn clear_message_set_wire_format(&mut self) {
    self.message_set_wire_format = __prelude::None
  }
  pub const NO_STANDARD_DESCRIPTOR_ACCESSOR_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const NO_STANDARD_DESCRIPTOR_ACCESSOR_DEFAULT: __prelude::bool = false;
  pub fn no_standard_descriptor_accessor(&self) -> __prelude::bool {
    self.no_standard_descriptor_accessor.unwrap_or(Self::NO_STANDARD_DESCRIPTOR_ACCESSOR_DEFAULT)
  }
  pub fn no_standard_descriptor_accessor_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.no_standard_descriptor_accessor.as_ref()
  }
  pub fn no_standard_descriptor_accessor_mut(&mut self) -> &mut __prelude::bool {
    self.no_standard_descriptor_accessor.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_no_standard_descriptor_accessor(&self) -> bool {
    self.no_standard_descriptor_accessor.is_some()
  }
  pub fn set_no_standard_descriptor_accessor(&mut self, value: __prelude::bool) {
    self.no_standard_descriptor_accessor = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_no_standard_descriptor_accessor(&mut self) -> __prelude::Option<__prelude::bool> {
    self.no_standard_descriptor_accessor.take()
  }
  pub fn clear_no_standard_descriptor_accessor(&mut self) {
    self.no_standard_descriptor_accessor = __prelude::None
  }
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const MAP_ENTRY_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(7) };
  pub const MAP_ENTRY_DEFAULT: __prelude::bool = false;
  pub fn map_entry(&self) -> __prelude::bool {
    self.map_entry.unwrap_or(Self::MAP_ENTRY_DEFAULT)
  }
  pub fn map_entry_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.map_entry.as_ref()
  }
  pub fn map_entry_mut(&mut self) -> &mut __prelude::bool {
    self.map_entry.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_map_entry(&self) -> bool {
    self.map_entry.is_some()
  }
  pub fn set_map_entry(&mut self, value: __prelude::bool) {
    self.map_entry = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_map_entry(&mut self) -> __prelude::Option<__prelude::bool> {
    self.map_entry.take()
  }
  pub fn clear_map_entry(&mut self) {
    self.map_entry = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FieldOptions {
  ctype: __prelude::Option<__file::field_options::CType>,
  packed: __prelude::Option<__prelude::bool>,
  jstype: __prelude::Option<__file::field_options::JSType>,
  lazy: __prelude::Option<__prelude::bool>,
  deprecated: __prelude::Option<__prelude::bool>,
  weak: __prelude::Option<__prelude::bool>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::FieldOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        8 => field.merge_value::<__prelude::pr::Enum<__file::field_options::CType>>(Self::CTYPE_NUMBER, self.ctype.get_or_insert_with(__prelude::Default::default))?,
        16 => field.merge_value::<__prelude::pr::Bool>(Self::PACKED_NUMBER, self.packed.get_or_insert_with(__prelude::Default::default))?,
        48 => field.merge_value::<__prelude::pr::Enum<__file::field_options::JSType>>(Self::JSTYPE_NUMBER, self.jstype.get_or_insert_with(__prelude::Default::default))?,
        40 => field.merge_value::<__prelude::pr::Bool>(Self::LAZY_NUMBER, self.lazy.get_or_insert_with(__prelude::Default::default))?,
        24 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        80 => field.merge_value::<__prelude::pr::Bool>(Self::WEAK_NUMBER, self.weak.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::FieldOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::FieldOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::FieldOptions { full_name: "google.protobuf.FieldOptions", name: "FieldOptions" });
impl self::FieldOptions {
  pub const CTYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const CTYPE_DEFAULT: __file::field_options::CType = __file::field_options::CType::STRING;
  pub fn ctype(&self) -> __file::field_options::CType {
    self.ctype.unwrap_or(Self::CTYPE_DEFAULT)
  }
  pub fn ctype_option(&self) -> __prelude::Option<&__file::field_options::CType> {
    self.ctype.as_ref()
  }
  pub fn ctype_mut(&mut self) -> &mut __file::field_options::CType {
    self.ctype.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_ctype(&self) -> bool {
    self.ctype.is_some()
  }
  pub fn set_ctype(&mut self, value: __file::field_options::CType) {
    self.ctype = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_ctype(&mut self) -> __prelude::Option<__file::field_options::CType> {
    self.ctype.take()
  }
  pub fn clear_ctype(&mut self) {
    self.ctype = __prelude::None
  }
  pub const PACKED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const PACKED_DEFAULT: __prelude::bool = false;
  pub fn packed(&self) -> __prelude::bool {
    self.packed.unwrap_or(Self::PACKED_DEFAULT)
  }
  pub fn packed_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.packed.as_ref()
  }
  pub fn packed_mut(&mut self) -> &mut __prelude::bool {
    self.packed.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_packed(&self) -> bool {
    self.packed.is_some()
  }
  pub fn set_packed(&mut self, value: __prelude::bool) {
    self.packed = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_packed(&mut self) -> __prelude::Option<__prelude::bool> {
    self.packed.take()
  }
  pub fn clear_packed(&mut self) {
    self.packed = __prelude::None
  }
  pub const JSTYPE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub const JSTYPE_DEFAULT: __file::field_options::JSType = __file::field_options::JSType::JS_NORMAL;
  pub fn jstype(&self) -> __file::field_options::JSType {
    self.jstype.unwrap_or(Self::JSTYPE_DEFAULT)
  }
  pub fn jstype_option(&self) -> __prelude::Option<&__file::field_options::JSType> {
    self.jstype.as_ref()
  }
  pub fn jstype_mut(&mut self) -> &mut __file::field_options::JSType {
    self.jstype.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_jstype(&self) -> bool {
    self.jstype.is_some()
  }
  pub fn set_jstype(&mut self, value: __file::field_options::JSType) {
    self.jstype = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_jstype(&mut self) -> __prelude::Option<__file::field_options::JSType> {
    self.jstype.take()
  }
  pub fn clear_jstype(&mut self) {
    self.jstype = __prelude::None
  }
  pub const LAZY_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub const LAZY_DEFAULT: __prelude::bool = false;
  pub fn lazy(&self) -> __prelude::bool {
    self.lazy.unwrap_or(Self::LAZY_DEFAULT)
  }
  pub fn lazy_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.lazy.as_ref()
  }
  pub fn lazy_mut(&mut self) -> &mut __prelude::bool {
    self.lazy.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_lazy(&self) -> bool {
    self.lazy.is_some()
  }
  pub fn set_lazy(&mut self, value: __prelude::bool) {
    self.lazy = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_lazy(&mut self) -> __prelude::Option<__prelude::bool> {
    self.lazy.take()
  }
  pub fn clear_lazy(&mut self) {
    self.lazy = __prelude::None
  }
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const WEAK_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(10) };
  pub const WEAK_DEFAULT: __prelude::bool = false;
  pub fn weak(&self) -> __prelude::bool {
    self.weak.unwrap_or(Self::WEAK_DEFAULT)
  }
  pub fn weak_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.weak.as_ref()
  }
  pub fn weak_mut(&mut self) -> &mut __prelude::bool {
    self.weak.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_weak(&self) -> bool {
    self.weak.is_some()
  }
  pub fn set_weak(&mut self, value: __prelude::bool) {
    self.weak = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_weak(&mut self) -> __prelude::Option<__prelude::bool> {
    self.weak.take()
  }
  pub fn clear_weak(&mut self) {
    self.weak = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
pub mod field_options {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct CType(pub i32);

  impl __prelude::Enum for CType { }
  impl __prelude::From<i32> for CType {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<CType> for i32 {
    fn from(x: CType) -> Self {
      x.0
    }
  }
  impl __prelude::Default for CType {
    fn default() -> Self {
      Self(0)
    }
  }
  impl CType {
    pub const STRING: Self = Self(0);
    pub const CORD: Self = Self(1);
    pub const STRING_PIECE: Self = Self(2);
  }
  impl __prelude::Debug for CType {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::STRING => f.write_str("STRING"),
        Self::CORD => f.write_str("CORD"),
        Self::STRING_PIECE => f.write_str("STRING_PIECE"),
        Self(x) => x.fmt(f),
      }
    }
  }
  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct JSType(pub i32);

  impl __prelude::Enum for JSType { }
  impl __prelude::From<i32> for JSType {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<JSType> for i32 {
    fn from(x: JSType) -> Self {
      x.0
    }
  }
  impl __prelude::Default for JSType {
    fn default() -> Self {
      Self(0)
    }
  }
  impl JSType {
    pub const JS_NORMAL: Self = Self(0);
    pub const JS_STRING: Self = Self(1);
    pub const JS_NUMBER: Self = Self(2);
  }
  impl __prelude::Debug for JSType {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::JS_NORMAL => f.write_str("JS_NORMAL"),
        Self::JS_STRING => f.write_str("JS_STRING"),
        Self::JS_NUMBER => f.write_str("JS_NUMBER"),
        Self(x) => x.fmt(f),
      }
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct OneofOptions {
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::OneofOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::OneofOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::OneofOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::OneofOptions { full_name: "google.protobuf.OneofOptions", name: "OneofOptions" });
impl self::OneofOptions {
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnumOptions {
  allow_alias: __prelude::Option<__prelude::bool>,
  deprecated: __prelude::Option<__prelude::bool>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::EnumOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        16 => field.merge_value::<__prelude::pr::Bool>(Self::ALLOW_ALIAS_NUMBER, self.allow_alias.get_or_insert_with(__prelude::Default::default))?,
        24 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::EnumOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::EnumOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::EnumOptions { full_name: "google.protobuf.EnumOptions", name: "EnumOptions" });
impl self::EnumOptions {
  pub const ALLOW_ALIAS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub const ALLOW_ALIAS_DEFAULT: __prelude::bool = false;
  pub fn allow_alias(&self) -> __prelude::bool {
    self.allow_alias.unwrap_or(Self::ALLOW_ALIAS_DEFAULT)
  }
  pub fn allow_alias_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.allow_alias.as_ref()
  }
  pub fn allow_alias_mut(&mut self) -> &mut __prelude::bool {
    self.allow_alias.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_allow_alias(&self) -> bool {
    self.allow_alias.is_some()
  }
  pub fn set_allow_alias(&mut self, value: __prelude::bool) {
    self.allow_alias = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_allow_alias(&mut self) -> __prelude::Option<__prelude::bool> {
    self.allow_alias.take()
  }
  pub fn clear_allow_alias(&mut self) {
    self.allow_alias = __prelude::None
  }
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnumValueOptions {
  deprecated: __prelude::Option<__prelude::bool>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::EnumValueOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        8 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::EnumValueOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::EnumValueOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::EnumValueOptions { full_name: "google.protobuf.EnumValueOptions", name: "EnumValueOptions" });
impl self::EnumValueOptions {
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ServiceOptions {
  deprecated: __prelude::Option<__prelude::bool>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::ServiceOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        264 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::ServiceOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::ServiceOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::ServiceOptions { full_name: "google.protobuf.ServiceOptions", name: "ServiceOptions" });
impl self::ServiceOptions {
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(33) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MethodOptions {
  deprecated: __prelude::Option<__prelude::bool>,
  idempotency_level: __prelude::Option<__file::method_options::IdempotencyLevel>,
  uninterpreted_option: __prelude::RepeatedField<__file::UninterpretedOption>,
  __extensions: __prelude::ExtensionSet<Self>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::MethodOptions {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        264 => field.merge_value::<__prelude::pr::Bool>(Self::DEPRECATED_NUMBER, self.deprecated.get_or_insert_with(__prelude::Default::default))?,
        272 => field.merge_value::<__prelude::pr::Enum<__file::method_options::IdempotencyLevel>>(Self::IDEMPOTENCY_LEVEL_NUMBER, self.idempotency_level.get_or_insert_with(__prelude::Default::default))?,
        7994 => field.add_entries_to::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &mut self.uninterpreted_option)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__extensions)?
            .or_try(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    builder = builder.add_fields(&self.__extensions)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::UninterpretedOption>>(Self::UNINTERPRETED_OPTION_NUMBER, &self.uninterpreted_option)?;
    output.write_fields(&self.__extensions)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::MethodOptions {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.uninterpreted_option) {
      return false;
    }
    true
  }
}
impl __prelude::ExtendableMessage for self::MethodOptions {
  fn extensions(&self) -> &__prelude::ExtensionSet<Self> {
    &self.__extensions
  }
  fn extensions_mut(&mut self) -> &mut __prelude::ExtensionSet<Self> {
    &mut self.__extensions
  }
}
__prelude::prefl::dbg_msg!(self::MethodOptions { full_name: "google.protobuf.MethodOptions", name: "MethodOptions" });
impl self::MethodOptions {
  pub const DEPRECATED_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(33) };
  pub const DEPRECATED_DEFAULT: __prelude::bool = false;
  pub fn deprecated(&self) -> __prelude::bool {
    self.deprecated.unwrap_or(Self::DEPRECATED_DEFAULT)
  }
  pub fn deprecated_option(&self) -> __prelude::Option<&__prelude::bool> {
    self.deprecated.as_ref()
  }
  pub fn deprecated_mut(&mut self) -> &mut __prelude::bool {
    self.deprecated.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_deprecated(&self) -> bool {
    self.deprecated.is_some()
  }
  pub fn set_deprecated(&mut self, value: __prelude::bool) {
    self.deprecated = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_deprecated(&mut self) -> __prelude::Option<__prelude::bool> {
    self.deprecated.take()
  }
  pub fn clear_deprecated(&mut self) {
    self.deprecated = __prelude::None
  }
  pub const IDEMPOTENCY_LEVEL_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(34) };
  pub const IDEMPOTENCY_LEVEL_DEFAULT: __file::method_options::IdempotencyLevel = __file::method_options::IdempotencyLevel::IDEMPOTENCY_UNKNOWN;
  pub fn idempotency_level(&self) -> __file::method_options::IdempotencyLevel {
    self.idempotency_level.unwrap_or(Self::IDEMPOTENCY_LEVEL_DEFAULT)
  }
  pub fn idempotency_level_option(&self) -> __prelude::Option<&__file::method_options::IdempotencyLevel> {
    self.idempotency_level.as_ref()
  }
  pub fn idempotency_level_mut(&mut self) -> &mut __file::method_options::IdempotencyLevel {
    self.idempotency_level.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_idempotency_level(&self) -> bool {
    self.idempotency_level.is_some()
  }
  pub fn set_idempotency_level(&mut self, value: __file::method_options::IdempotencyLevel) {
    self.idempotency_level = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_idempotency_level(&mut self) -> __prelude::Option<__file::method_options::IdempotencyLevel> {
    self.idempotency_level.take()
  }
  pub fn clear_idempotency_level(&mut self) {
    self.idempotency_level = __prelude::None
  }
  pub const UNINTERPRETED_OPTION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(999) };
  pub fn uninterpreted_option(&self) -> &__prelude::RepeatedField<__file::UninterpretedOption> {
    &self.uninterpreted_option
  }
  pub fn uninterpreted_option_mut(&mut self) -> &mut __prelude::RepeatedField<__file::UninterpretedOption> {
    &mut self.uninterpreted_option
  }
}
pub mod method_options {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
  pub struct IdempotencyLevel(pub i32);

  impl __prelude::Enum for IdempotencyLevel { }
  impl __prelude::From<i32> for IdempotencyLevel {
    fn from(x: i32) -> Self {
      Self(x)
    }
  }
  impl __prelude::From<IdempotencyLevel> for i32 {
    fn from(x: IdempotencyLevel) -> Self {
      x.0
    }
  }
  impl __prelude::Default for IdempotencyLevel {
    fn default() -> Self {
      Self(0)
    }
  }
  impl IdempotencyLevel {
    pub const IDEMPOTENCY_UNKNOWN: Self = Self(0);
    pub const NO_SIDE_EFFECTS: Self = Self(1);
    pub const IDEMPOTENT: Self = Self(2);
  }
  impl __prelude::Debug for IdempotencyLevel {
    fn fmt(&self, f: &mut __prelude::Formatter) -> __prelude::fmt::Result {
      #[allow(unreachable_patterns)]
      match *self {
        Self::IDEMPOTENCY_UNKNOWN => f.write_str("IDEMPOTENCY_UNKNOWN"),
        Self::NO_SIDE_EFFECTS => f.write_str("NO_SIDE_EFFECTS"),
        Self::IDEMPOTENT => f.write_str("IDEMPOTENT"),
        Self(x) => x.fmt(f),
      }
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UninterpretedOption {
  name: __prelude::RepeatedField<__file::uninterpreted_option::NamePart>,
  identifier_value: __prelude::Option<__prelude::String>,
  positive_int_value: __prelude::Option<__prelude::u64>,
  negative_int_value: __prelude::Option<__prelude::i64>,
  double_value: __prelude::Option<__prelude::f64>,
  string_value: __prelude::Option<__prelude::ByteVec>,
  aggregate_value: __prelude::Option<__prelude::String>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::UninterpretedOption {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        18 => field.add_entries_to::<_, __prelude::pr::Message<__file::uninterpreted_option::NamePart>>(Self::NAME_NUMBER, &mut self.name)?,
        26 => field.merge_value::<__prelude::pr::String>(Self::IDENTIFIER_VALUE_NUMBER, self.identifier_value.get_or_insert_with(__prelude::Default::default))?,
        32 => field.merge_value::<__prelude::pr::Uint64>(Self::POSITIVE_INT_VALUE_NUMBER, self.positive_int_value.get_or_insert_with(__prelude::Default::default))?,
        40 => field.merge_value::<__prelude::pr::Int64>(Self::NEGATIVE_INT_VALUE_NUMBER, self.negative_int_value.get_or_insert_with(__prelude::Default::default))?,
        49 => field.merge_value::<__prelude::pr::Double>(Self::DOUBLE_VALUE_NUMBER, self.double_value.get_or_insert_with(__prelude::Default::default))?,
        58 => field.merge_value::<__prelude::pr::Bytes<__prelude::ByteVec>>(Self::STRING_VALUE_NUMBER, self.string_value.get_or_insert_with(__prelude::Default::default))?,
        66 => field.merge_value::<__prelude::pr::String>(Self::AGGREGATE_VALUE_NUMBER, self.aggregate_value.get_or_insert_with(__prelude::Default::default))?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::uninterpreted_option::NamePart>>(Self::NAME_NUMBER, &self.name)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::uninterpreted_option::NamePart>>(Self::NAME_NUMBER, &self.name)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::UninterpretedOption {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.name) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::UninterpretedOption { full_name: "google.protobuf.UninterpretedOption", name: "UninterpretedOption" });
impl self::UninterpretedOption {
  pub const NAME_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
  pub fn name(&self) -> &__prelude::RepeatedField<__file::uninterpreted_option::NamePart> {
    &self.name
  }
  pub fn name_mut(&mut self) -> &mut __prelude::RepeatedField<__file::uninterpreted_option::NamePart> {
    &mut self.name
  }
  pub const IDENTIFIER_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
  pub const IDENTIFIER_VALUE_DEFAULT: &'static __prelude::str = "";
  pub fn identifier_value(&self) -> &__prelude::str {
    self.identifier_value.as_ref().map_or(Self::IDENTIFIER_VALUE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn identifier_value_option(&self) -> __prelude::Option<&__prelude::String> {
    self.identifier_value.as_ref()
  }
  pub fn identifier_value_mut(&mut self) -> &mut __prelude::String {
    self.identifier_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_identifier_value(&self) -> bool {
    self.identifier_value.is_some()
  }
  pub fn set_identifier_value(&mut self, value: __prelude::String) {
    self.identifier_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_identifier_value(&mut self) -> __prelude::Option<__prelude::String> {
    self.identifier_value.take()
  }
  pub fn clear_identifier_value(&mut self) {
    self.identifier_value = __prelude::None
  }
  pub const POSITIVE_INT_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
  pub const POSITIVE_INT_VALUE_DEFAULT: __prelude::u64 = 0;
  pub fn positive_int_value(&self) -> __prelude::u64 {
    self.positive_int_value.unwrap_or(Self::POSITIVE_INT_VALUE_DEFAULT)
  }
  pub fn positive_int_value_option(&self) -> __prelude::Option<&__prelude::u64> {
    self.positive_int_value.as_ref()
  }
  pub fn positive_int_value_mut(&mut self) -> &mut __prelude::u64 {
    self.positive_int_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_positive_int_value(&self) -> bool {
    self.positive_int_value.is_some()
  }
  pub fn set_positive_int_value(&mut self, value: __prelude::u64) {
    self.positive_int_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_positive_int_value(&mut self) -> __prelude::Option<__prelude::u64> {
    self.positive_int_value.take()
  }
  pub fn clear_positive_int_value(&mut self) {
    self.positive_int_value = __prelude::None
  }
  pub const NEGATIVE_INT_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(5) };
  pub const NEGATIVE_INT_VALUE_DEFAULT: __prelude::i64 = 0;
  pub fn negative_int_value(&self) -> __prelude::i64 {
    self.negative_int_value.unwrap_or(Self::NEGATIVE_INT_VALUE_DEFAULT)
  }
  pub fn negative_int_value_option(&self) -> __prelude::Option<&__prelude::i64> {
    self.negative_int_value.as_ref()
  }
  pub fn negative_int_value_mut(&mut self) -> &mut __prelude::i64 {
    self.negative_int_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_negative_int_value(&self) -> bool {
    self.negative_int_value.is_some()
  }
  pub fn set_negative_int_value(&mut self, value: __prelude::i64) {
    self.negative_int_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_negative_int_value(&mut self) -> __prelude::Option<__prelude::i64> {
    self.negative_int_value.take()
  }
  pub fn clear_negative_int_value(&mut self) {
    self.negative_int_value = __prelude::None
  }
  pub const DOUBLE_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
  pub const DOUBLE_VALUE_DEFAULT: __prelude::f64 = 0.000000;
  pub fn double_value(&self) -> __prelude::f64 {
    self.double_value.unwrap_or(Self::DOUBLE_VALUE_DEFAULT)
  }
  pub fn double_value_option(&self) -> __prelude::Option<&__prelude::f64> {
    self.double_value.as_ref()
  }
  pub fn double_value_mut(&mut self) -> &mut __prelude::f64 {
    self.double_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_double_value(&self) -> bool {
    self.double_value.is_some()
  }
  pub fn set_double_value(&mut self, value: __prelude::f64) {
    self.double_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_double_value(&mut self) -> __prelude::Option<__prelude::f64> {
    self.double_value.take()
  }
  pub fn clear_double_value(&mut self) {
    self.double_value = __prelude::None
  }
  pub const STRING_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(7) };
  pub const STRING_VALUE_DEFAULT: &'static [__prelude::u8] = b"";
  pub fn string_value(&self) -> &[__prelude::u8] {
    self.string_value.as_ref().map_or(Self::STRING_VALUE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn string_value_option(&self) -> __prelude::Option<&__prelude::ByteVec> {
    self.string_value.as_ref()
  }
  pub fn string_value_mut(&mut self) -> &mut __prelude::ByteVec {
    self.string_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_string_value(&self) -> bool {
    self.string_value.is_some()
  }
  pub fn set_string_value(&mut self, value: __prelude::ByteVec) {
    self.string_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_string_value(&mut self) -> __prelude::Option<__prelude::ByteVec> {
    self.string_value.take()
  }
  pub fn clear_string_value(&mut self) {
    self.string_value = __prelude::None
  }
  pub const AGGREGATE_VALUE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(8) };
  pub const AGGREGATE_VALUE_DEFAULT: &'static __prelude::str = "";
  pub fn aggregate_value(&self) -> &__prelude::str {
    self.aggregate_value.as_ref().map_or(Self::AGGREGATE_VALUE_DEFAULT, __prelude::AsRef::as_ref)
  }
  pub fn aggregate_value_option(&self) -> __prelude::Option<&__prelude::String> {
    self.aggregate_value.as_ref()
  }
  pub fn aggregate_value_mut(&mut self) -> &mut __prelude::String {
    self.aggregate_value.get_or_insert_with(__prelude::Default::default)
  }
  pub fn has_aggregate_value(&self) -> bool {
    self.aggregate_value.is_some()
  }
  pub fn set_aggregate_value(&mut self, value: __prelude::String) {
    self.aggregate_value = __prelude::Some(__prelude::From::from(value))
  }
  pub fn take_aggregate_value(&mut self) -> __prelude::Option<__prelude::String> {
    self.aggregate_value.take()
  }
  pub fn clear_aggregate_value(&mut self) {
    self.aggregate_value = __prelude::None
  }
}
pub mod uninterpreted_option {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct NamePart {
    name_part: __prelude::Option<__prelude::String>,
    is_extension: __prelude::Option<__prelude::bool>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::NamePart {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          10 => field.merge_value::<__prelude::pr::String>(Self::NAME_PART_NUMBER, self.name_part.get_or_insert_with(__prelude::Default::default))?,
          16 => field.merge_value::<__prelude::pr::Bool>(Self::IS_EXTENSION_NUMBER, self.is_extension.get_or_insert_with(__prelude::Default::default))?,
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::NamePart {
    fn is_initialized(&self) -> bool {
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::NamePart { full_name: "google.protobuf.UninterpretedOption.NamePart", name: "NamePart" });
  impl self::NamePart {
    pub const NAME_PART_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub const NAME_PART_DEFAULT: &'static __prelude::str = "";
    pub fn name_part(&self) -> &__prelude::str {
      self.name_part.as_ref().map_or(Self::NAME_PART_DEFAULT, __prelude::AsRef::as_ref)
    }
    pub fn name_part_option(&self) -> __prelude::Option<&__prelude::String> {
      self.name_part.as_ref()
    }
    pub fn name_part_mut(&mut self) -> &mut __prelude::String {
      self.name_part.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_name_part(&self) -> bool {
      self.name_part.is_some()
    }
    pub fn set_name_part(&mut self, value: __prelude::String) {
      self.name_part = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_name_part(&mut self) -> __prelude::Option<__prelude::String> {
      self.name_part.take()
    }
    pub fn clear_name_part(&mut self) {
      self.name_part = __prelude::None
    }
    pub const IS_EXTENSION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub const IS_EXTENSION_DEFAULT: __prelude::bool = false;
    pub fn is_extension(&self) -> __prelude::bool {
      self.is_extension.unwrap_or(Self::IS_EXTENSION_DEFAULT)
    }
    pub fn is_extension_option(&self) -> __prelude::Option<&__prelude::bool> {
      self.is_extension.as_ref()
    }
    pub fn is_extension_mut(&mut self) -> &mut __prelude::bool {
      self.is_extension.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_is_extension(&self) -> bool {
      self.is_extension.is_some()
    }
    pub fn set_is_extension(&mut self, value: __prelude::bool) {
      self.is_extension = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_is_extension(&mut self) -> __prelude::Option<__prelude::bool> {
      self.is_extension.take()
    }
    pub fn clear_is_extension(&mut self) {
      self.is_extension = __prelude::None
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SourceCodeInfo {
  location: __prelude::RepeatedField<__file::source_code_info::Location>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::SourceCodeInfo {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.add_entries_to::<_, __prelude::pr::Message<__file::source_code_info::Location>>(Self::LOCATION_NUMBER, &mut self.location)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::source_code_info::Location>>(Self::LOCATION_NUMBER, &self.location)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::source_code_info::Location>>(Self::LOCATION_NUMBER, &self.location)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::SourceCodeInfo {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.location) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::SourceCodeInfo { full_name: "google.protobuf.SourceCodeInfo", name: "SourceCodeInfo" });
impl self::SourceCodeInfo {
  pub const LOCATION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub fn location(&self) -> &__prelude::RepeatedField<__file::source_code_info::Location> {
    &self.location
  }
  pub fn location_mut(&mut self) -> &mut __prelude::RepeatedField<__file::source_code_info::Location> {
    &mut self.location
  }
}
pub mod source_code_info {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct Location {
    path: __prelude::RepeatedField<__prelude::i32>,
    span: __prelude::RepeatedField<__prelude::i32>,
    leading_comments: __prelude::Option<__prelude::String>,
    trailing_comments: __prelude::Option<__prelude::String>,
    leading_detached_comments: __prelude::RepeatedField<__prelude::String>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::Location {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          10 => field.add_entries_to::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &mut self.path)?,
          8 => field.add_entries_to::<_, __prelude::pr::Int32>(Self::PATH_NUMBER, &mut self.path)?,
          18 => field.add_entries_to::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::SPAN_NUMBER, &mut self.span)?,
          16 => field.add_entries_to::<_, __prelude::pr::Int32>(Self::SPAN_NUMBER, &mut self.span)?,
          26 => field.merge_value::<__prelude::pr::String>(Self::LEADING_COMMENTS_NUMBER, self.leading_comments.get_or_insert_with(__prelude::Default::default))?,
          34 => field.merge_value::<__prelude::pr::String>(Self::TRAILING_COMMENTS_NUMBER, self.trailing_comments.get_or_insert_with(__prelude::Default::default))?,
          50 => field.add_entries_to::<_, __prelude::pr::String>(Self::LEADING_DETACHED_COMMENTS_NUMBER, &mut self.leading_detached_comments)?,
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &self.path)?;
      builder = builder.add_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::SPAN_NUMBER, &self.span)?;
      builder = builder.add_values::<_, __prelude::pr::String>(Self::LEADING_DETACHED_COMMENTS_NUMBER, &self.leading_detached_comments)?;
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &self.path)?;
      output.write_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::SPAN_NUMBER, &self.span)?;
      output.write_values::<_, __prelude::pr::String>(Self::LEADING_DETACHED_COMMENTS_NUMBER, &self.leading_detached_comments)?;
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::Location {
    fn is_initialized(&self) -> bool {
      if !__prelude::p::is_initialized(&self.path) {
        return false;
      }
      if !__prelude::p::is_initialized(&self.span) {
        return false;
      }
      if !__prelude::p::is_initialized(&self.leading_detached_comments) {
        return false;
      }
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::Location { full_name: "google.protobuf.SourceCodeInfo.Location", name: "Location" });
  impl self::Location {
    pub const PATH_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub fn path(&self) -> &__prelude::RepeatedField<__prelude::i32> {
      &self.path
    }
    pub fn path_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::i32> {
      &mut self.path
    }
    pub const SPAN_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub fn span(&self) -> &__prelude::RepeatedField<__prelude::i32> {
      &self.span
    }
    pub fn span_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::i32> {
      &mut self.span
    }
    pub const LEADING_COMMENTS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
    pub const LEADING_COMMENTS_DEFAULT: &'static __prelude::str = "";
    pub fn leading_comments(&self) -> &__prelude::str {
      self.leading_comments.as_ref().map_or(Self::LEADING_COMMENTS_DEFAULT, __prelude::AsRef::as_ref)
    }
    pub fn leading_comments_option(&self) -> __prelude::Option<&__prelude::String> {
      self.leading_comments.as_ref()
    }
    pub fn leading_comments_mut(&mut self) -> &mut __prelude::String {
      self.leading_comments.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_leading_comments(&self) -> bool {
      self.leading_comments.is_some()
    }
    pub fn set_leading_comments(&mut self, value: __prelude::String) {
      self.leading_comments = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_leading_comments(&mut self) -> __prelude::Option<__prelude::String> {
      self.leading_comments.take()
    }
    pub fn clear_leading_comments(&mut self) {
      self.leading_comments = __prelude::None
    }
    pub const TRAILING_COMMENTS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
    pub const TRAILING_COMMENTS_DEFAULT: &'static __prelude::str = "";
    pub fn trailing_comments(&self) -> &__prelude::str {
      self.trailing_comments.as_ref().map_or(Self::TRAILING_COMMENTS_DEFAULT, __prelude::AsRef::as_ref)
    }
    pub fn trailing_comments_option(&self) -> __prelude::Option<&__prelude::String> {
      self.trailing_comments.as_ref()
    }
    pub fn trailing_comments_mut(&mut self) -> &mut __prelude::String {
      self.trailing_comments.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_trailing_comments(&self) -> bool {
      self.trailing_comments.is_some()
    }
    pub fn set_trailing_comments(&mut self, value: __prelude::String) {
      self.trailing_comments = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_trailing_comments(&mut self) -> __prelude::Option<__prelude::String> {
      self.trailing_comments.take()
    }
    pub fn clear_trailing_comments(&mut self) {
      self.trailing_comments = __prelude::None
    }
    pub const LEADING_DETACHED_COMMENTS_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(6) };
    pub fn leading_detached_comments(&self) -> &__prelude::RepeatedField<__prelude::String> {
      &self.leading_detached_comments
    }
    pub fn leading_detached_comments_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::String> {
      &mut self.leading_detached_comments
    }
  }
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GeneratedCodeInfo {
  annotation: __prelude::RepeatedField<__file::generated_code_info::Annotation>,
  __unknown_fields: __prelude::UnknownFieldSet,
}
impl __prelude::Message for self::GeneratedCodeInfo {
  fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
    while let __prelude::Some(field) = input.read_field()? {
      match field.tag() {
        10 => field.add_entries_to::<_, __prelude::pr::Message<__file::generated_code_info::Annotation>>(Self::ANNOTATION_NUMBER, &mut self.annotation)?,
        _ => 
          field
            .check_and_try_add_field_to(&mut self.__unknown_fields)?
            .or_skip()?
      }
    }
    __prelude::Ok(())
  }
  fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
    let mut builder = __prelude::pio::LengthBuilder::new();
    builder = builder.add_values::<_, __prelude::pr::Message<__file::generated_code_info::Annotation>>(Self::ANNOTATION_NUMBER, &self.annotation)?;
    builder = builder.add_fields(&self.__unknown_fields)?;
    __prelude::Some(builder.build())}
  fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
    output.write_values::<_, __prelude::pr::Message<__file::generated_code_info::Annotation>>(Self::ANNOTATION_NUMBER, &self.annotation)?;
    output.write_fields(&self.__unknown_fields)?;
    __prelude::Ok(())
  }
  fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
    &self.__unknown_fields
  }
  fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
    &mut self.__unknown_fields
  }
}
impl __prelude::Initializable for self::GeneratedCodeInfo {
  fn is_initialized(&self) -> bool {
    if !__prelude::p::is_initialized(&self.annotation) {
      return false;
    }
    true
  }
}
__prelude::prefl::dbg_msg!(self::GeneratedCodeInfo { full_name: "google.protobuf.GeneratedCodeInfo", name: "GeneratedCodeInfo" });
impl self::GeneratedCodeInfo {
  pub const ANNOTATION_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
  pub fn annotation(&self) -> &__prelude::RepeatedField<__file::generated_code_info::Annotation> {
    &self.annotation
  }
  pub fn annotation_mut(&mut self) -> &mut __prelude::RepeatedField<__file::generated_code_info::Annotation> {
    &mut self.annotation
  }
}
pub mod generated_code_info {
  pub(self) use super::__file;
  pub(self) use ::protrust::gen_prelude as __prelude;

  #[derive(Clone, Debug, PartialEq, Default)]
  pub struct Annotation {
    path: __prelude::RepeatedField<__prelude::i32>,
    source_file: __prelude::Option<__prelude::String>,
    begin: __prelude::Option<__prelude::i32>,
    end: __prelude::Option<__prelude::i32>,
    __unknown_fields: __prelude::UnknownFieldSet,
  }
  impl __prelude::Message for self::Annotation {
    fn merge_from<T: __prelude::Input>(&mut self, input: &mut __prelude::CodedReader<T>) -> __prelude::read::Result<()> {
      while let __prelude::Some(field) = input.read_field()? {
        match field.tag() {
          10 => field.add_entries_to::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &mut self.path)?,
          8 => field.add_entries_to::<_, __prelude::pr::Int32>(Self::PATH_NUMBER, &mut self.path)?,
          18 => field.merge_value::<__prelude::pr::String>(Self::SOURCE_FILE_NUMBER, self.source_file.get_or_insert_with(__prelude::Default::default))?,
          24 => field.merge_value::<__prelude::pr::Int32>(Self::BEGIN_NUMBER, self.begin.get_or_insert_with(__prelude::Default::default))?,
          32 => field.merge_value::<__prelude::pr::Int32>(Self::END_NUMBER, self.end.get_or_insert_with(__prelude::Default::default))?,
          _ => 
            field
              .check_and_try_add_field_to(&mut self.__unknown_fields)?
              .or_skip()?
        }
      }
      __prelude::Ok(())
    }
    fn calculate_size(&self) -> __prelude::Option<__prelude::Length> {
      let mut builder = __prelude::pio::LengthBuilder::new();
      builder = builder.add_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &self.path)?;
      builder = builder.add_fields(&self.__unknown_fields)?;
      __prelude::Some(builder.build())}
    fn write_to<T: __prelude::Output>(&self, output: &mut __prelude::CodedWriter<T>) -> __prelude::write::Result {
      output.write_values::<_, __prelude::pr::Packed<__prelude::pr::Int32>>(Self::PATH_NUMBER, &self.path)?;
      output.write_fields(&self.__unknown_fields)?;
      __prelude::Ok(())
    }
    fn unknown_fields(&self) -> &__prelude::UnknownFieldSet {
      &self.__unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut __prelude::UnknownFieldSet {
      &mut self.__unknown_fields
    }
  }
  impl __prelude::Initializable for self::Annotation {
    fn is_initialized(&self) -> bool {
      if !__prelude::p::is_initialized(&self.path) {
        return false;
      }
      true
    }
  }
  __prelude::prefl::dbg_msg!(self::Annotation { full_name: "google.protobuf.GeneratedCodeInfo.Annotation", name: "Annotation" });
  impl self::Annotation {
    pub const PATH_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(1) };
    pub fn path(&self) -> &__prelude::RepeatedField<__prelude::i32> {
      &self.path
    }
    pub fn path_mut(&mut self) -> &mut __prelude::RepeatedField<__prelude::i32> {
      &mut self.path
    }
    pub const SOURCE_FILE_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(2) };
    pub const SOURCE_FILE_DEFAULT: &'static __prelude::str = "";
    pub fn source_file(&self) -> &__prelude::str {
      self.source_file.as_ref().map_or(Self::SOURCE_FILE_DEFAULT, __prelude::AsRef::as_ref)
    }
    pub fn source_file_option(&self) -> __prelude::Option<&__prelude::String> {
      self.source_file.as_ref()
    }
    pub fn source_file_mut(&mut self) -> &mut __prelude::String {
      self.source_file.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_source_file(&self) -> bool {
      self.source_file.is_some()
    }
    pub fn set_source_file(&mut self, value: __prelude::String) {
      self.source_file = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_source_file(&mut self) -> __prelude::Option<__prelude::String> {
      self.source_file.take()
    }
    pub fn clear_source_file(&mut self) {
      self.source_file = __prelude::None
    }
    pub const BEGIN_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(3) };
    pub const BEGIN_DEFAULT: __prelude::i32 = 0;
    pub fn begin(&self) -> __prelude::i32 {
      self.begin.unwrap_or(Self::BEGIN_DEFAULT)
    }
    pub fn begin_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.begin.as_ref()
    }
    pub fn begin_mut(&mut self) -> &mut __prelude::i32 {
      self.begin.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_begin(&self) -> bool {
      self.begin.is_some()
    }
    pub fn set_begin(&mut self, value: __prelude::i32) {
      self.begin = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_begin(&mut self) -> __prelude::Option<__prelude::i32> {
      self.begin.take()
    }
    pub fn clear_begin(&mut self) {
      self.begin = __prelude::None
    }
    pub const END_NUMBER: __prelude::FieldNumber = unsafe { __prelude::FieldNumber::new_unchecked(4) };
    pub const END_DEFAULT: __prelude::i32 = 0;
    pub fn end(&self) -> __prelude::i32 {
      self.end.unwrap_or(Self::END_DEFAULT)
    }
    pub fn end_option(&self) -> __prelude::Option<&__prelude::i32> {
      self.end.as_ref()
    }
    pub fn end_mut(&mut self) -> &mut __prelude::i32 {
      self.end.get_or_insert_with(__prelude::Default::default)
    }
    pub fn has_end(&self) -> bool {
      self.end.is_some()
    }
    pub fn set_end(&mut self, value: __prelude::i32) {
      self.end = __prelude::Some(__prelude::From::from(value))
    }
    pub fn take_end(&mut self) -> __prelude::Option<__prelude::i32> {
      self.end.take()
    }
    pub fn clear_end(&mut self) {
      self.end = __prelude::None
    }
  }
}
