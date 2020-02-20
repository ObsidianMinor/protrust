#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "reflect"))]
macro_rules! dbg_msg {
    ($type:ty {
        full_name: $full_name:literal,
        name: $name:literal $(,)?
    }) => {
        impl $crate::reflect::DebugMessage for $type {
            fn full_name() -> &'static str {
                $full_name
            }
            fn name() -> &'static str {
                $name
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "reflect")]
macro_rules! dbg_msg {
    ($($tt:tt)*) => { };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "reflect"))]
macro_rules! gen_mod {
    ($($tt:tt)*) => { };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "reflect")]
macro_rules! gen_mod {
    ($($tt:tt)*) => {
        $crate::export::macros::gen_mod_reflect!($crate: $($tt)*)
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "reflect"))]
macro_rules! file {
    ($($tt:tt)*) => { };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "reflect")]
macro_rules! file {
    ($accessor:ident: $pool:path => $name:literal) => {
        use $crate::reflect::FileDescriptor;

        pub fn $accessor() -> &'static FileDescriptor<'static> {
            $pool.find_file_by_name($name).unwrap()
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "reflect"))]
macro_rules! msg_type {
    ($($tt:tt)*) => { };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "reflect")]
macro_rules! msg_type {
    ($type:ty: $access:expr) => {
        impl $crate::reflect::MessageType for $type {
            fn descriptor() -> &'static $crate::reflect::MessageDescriptor<'static> {
                $access
            }
        }
    };
}