use snafu::prelude::*;

/// Error type for decoding errors.
///
/// This is used to report errors while decoding a class file.
#[derive(Debug, Snafu)]
pub enum DecodingError {
    #[snafu(display("Invalid this_class name, at entry {}: {}", index, message.as_deref().unwrap_or("<no context provided>")))]
    InvalidThisClass { index: usize, message: Option<String> },

    #[snafu(display("Invalid super_class name, at entry {}: {}", index, message.as_deref().unwrap_or("<no context provided>")))]
    InvalidSuperClass { index: usize, message: Option<String> },

    #[snafu(display("Invalid interface name, at entry {}: {}", index, message.as_deref().unwrap_or("<no context provided>")))]
    InvalidInterface { index: usize, message: Option<String> },

    #[snafu(display("Unexpected error, causes:\n{:?}", context.as_deref().unwrap_or("<no context provided>")))]
    Unknown { context: Option<String> },
}