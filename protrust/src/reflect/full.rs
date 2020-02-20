use super::DebugMessage;

impl<T: MessageType> DebugMessage for T {
    fn full_name() -> &'static str {
        T::descriptor().full_name()
    }
    fn name() -> &'static str {
        T::descriptor().name()
    }
}

pub trait MessageType {
    fn descriptor() -> &'static MessageDescriptor<'static>;
}

pub trait EnumType {
    fn descriptor() -> &'static EnumDescriptor<'static>;
}