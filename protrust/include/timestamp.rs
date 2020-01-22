use protrust::{UnknownFieldSet, Mergable, Message, raw};
use protrust::io::{read, write, CodedReader, Input, CodedWriter, Output, FieldNumber, Tag, LengthBuilder};
use protrust::raw as r;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Timestamp {
    seconds: i64,
    nanos: i32,
    unknown_fields: UnknownFieldSet,
}

impl Timestamp {
    pub const SECONDS_NUMBER: FieldNumber = unsafe { FieldNumber::new_unchecked(1) };
    pub fn seconds(&self) -> &i64 {
        &self.seconds
    }
    pub fn seconds_mut(&mut self) -> &mut i64 {
        &mut self.seconds
    }

    pub const NANOS_NUMBER: FieldNumber = unsafe { FieldNumber::new_unchecked(2) };
    pub fn nanos(&self) -> &i32 {
        &self.nanos
    }
    pub fn nanos_mut(&mut self) -> &mut i32 {
        &mut self.nanos
    }
}

impl Mergable for Timestamp {
    fn merge(&mut self, other: &Self) {
        if other.seconds != 0 {
            self.seconds = other.seconds;
        }
        if other.nanos != 0 {
            self.nanos = other.nanos;
        }
        self.unknown_fields.merge(&other.unknown_fields);
    }
}

impl Message for Timestamp {
    fn merge_from<T: Input>(&mut self, input: &mut CodedReader<T>) -> read::Result<()> {
        while let Some(field) = input.read_field()? {
            match field.tag() {
                8 => field.read_value(unsafe { Tag::new_unchecked(8) }, |input| input.merge_value::<r::Int64>(&mut self.seconds))?,
                16 => field.read_value(unsafe { Tag::new_unchecked(16) }, |input| input.merge_value::<r::Int32>(&mut self.nanos))?,
                _ => field.check_and_read_value(|input| input.try_add_field_to(&mut self.unknown_fields)?.or_skip())?,
            }
        }
        Ok(())
    }
    fn calculate_size(&self, mut builder: LengthBuilder) -> Option<LengthBuilder> {
        if self.seconds != 0 {
            builder =
                builder.add_field::<r::Int64>(Self::SECONDS_NUMBER, self.seconds())?;
        }
        if self.nanos != 0 {
            builder =
                builder.add_field::<r::Int32>(Self::NANOS_NUMBER, self.nanos())?;
        }
        builder =
            builder.add_fields(&self.unknown_fields)?;

        Some(builder)
    }
    fn write_to<T: Output>(&self, output: &mut CodedWriter<T>) -> write::Result {
        if self.seconds != 0 {
            output.write_field::<r::Int64>(Self::SECONDS_NUMBER, &self.seconds)?;
        }
        if self.nanos != 0 {
            output.write_field::<r::Int32>(Self::NANOS_NUMBER, &self.nanos)?;
        }
        output.write_fields(&self.unknown_fields)?;
        Ok(())
    }
    fn is_initialized(&self) -> bool {
        true
    }

    fn unknown_fields(&self) -> &UnknownFieldSet {
        &self.unknown_fields
    }
    fn unknown_fields_mut(&mut self) -> &mut UnknownFieldSet {
        &mut self.unknown_fields
    }

    fn new() -> Self {
        Default::default()
    }
}