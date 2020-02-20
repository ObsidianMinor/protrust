#[cfg(feature = "reflect")]
mod full;
mod macros;

#[cfg(feature = "reflect")]
pub use full::*;

#[doc(hidden)]
pub use crate::{dbg_msg, gen_mod, file, msg_type};

/// Provides basic static type information about a protobuf message type.
/// 
/// This trait is provided with or without reflection.
pub trait DebugMessage {
    /// The full name of the message without a preceeding dot.
    fn full_name() -> &'static str;
    /// The name of the message type.
    fn name() -> &'static str;
}