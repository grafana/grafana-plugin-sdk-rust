use std::fmt;

/// The source of an error.
///
/// This is used to indicate whether the error occurred in the plugin or downstream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorSource {
    /// The error occurred in the plugin.
    #[default]
    Plugin,
    /// The error occurred downstream of the plugin.
    Downstream,
}

impl fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plugin => f.write_str("plugin"),
            Self::Downstream => f.write_str("downstream"),
        }
    }
}
