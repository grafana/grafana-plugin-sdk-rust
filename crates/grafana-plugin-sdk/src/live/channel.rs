/*! Identifiers for 'live' channels.

See the [channel guide] for more information.

[channel guide]: https://grafana.com/docs/grafana/latest/live/live-channel/
*/
use std::{fmt, str::FromStr};

use itertools::Itertools;
use serde::Serialize;
use thiserror::Error;

/// The maximum length of a channel when represented as a string.
pub const MAX_CHANNEL_LENGTH: usize = 160;

/// The error returned when parsing a channel.
#[derive(Debug, Error, Serialize)]
#[non_exhaustive]
pub enum Error {
    /// The channel was empty.
    #[error("Channel must not be empty")]
    Empty,
    /// The channel exceeded the maximum length of [`MAX_CHANNEL_LENGTH`].
    #[error("Channel exceeds max length of {MAX_CHANNEL_LENGTH}")]
    ExceedsMaxLength,
    /// The channel did not contain a scope segment.
    #[error("Missing scope")]
    MissingScope,
    /// The channel did not contain a namespace segment.
    #[error("Missing namespace")]
    MissingNamespace,
    /// The channel did not contain a path segment.
    #[error("Missing path")]
    MissingPath,
    /// The channel's scope was invalid.
    #[error(r#"Invalid scope {0}; must be one of "grafana", "plugin", "ds", "stream""#)]
    InvalidScope(String),
    /// The channel's namespace was invalid.
    #[error("Invalid namespace {full}; must only contain ASCII alphanumeric, hyphens, and underscores. Found {invalid}")]
    InvalidNamespace {
        /// The full namespace string supplied.
        full: String,
        /// The unique invalid characters detected in the invalid namespace.
        invalid: String,
    },
    /// The channel's path was invalid.
    #[error(
        "Invalid path {full}; must only contain ASCII alphanumeric and any of '_-=/.'. Found {invalid}"
    )]
    InvalidPath {
        /// The full namespace string supplied.
        full: String,
        /// The unique invalid characters detected in the invalid namespace.
        invalid: String,
    },
}

type Result<T> = std::result::Result<T, Error>;

/// The scope of a channel.
///
/// This determines the purpose of a channel in Grafana Live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Built-in real-time features of Grafana core.
    Grafana,
    /// Passes control to a plugin.
    Plugin,
    /// Passes control to a datasource plugin.
    Datasource,
    /// A managed data frame stream.
    Stream,
}

impl FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "grafana" => Self::Grafana,
            "plugin" => Self::Plugin,
            "ds" => Self::Datasource,
            "stream" => Self::Stream,
            invalid => return Err(Error::InvalidScope(invalid.to_string())),
        })
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Grafana => "grafana",
            Self::Plugin => "plugin",
            Self::Datasource => "ds",
            Self::Stream => "stream",
        };
        write!(f, "{}", s)
    }
}

/// The namespace of a channel.
///
/// This has a different meaning depending on the scope of the channel:
/// - when scope is [`Scope::Grafana`], namespace is a "feature"
/// - when scope is [`Scope::Plugin`], namespace is the plugin name
/// - when scope is [`Scope::Datasource`], namespace is the datasource `uid`.
/// - when scope is [`Scope::Stream`], namespace is the stream ID.
///
/// Namespaces must only contain ASCII alphanumeric characters or any of `_-`.
/// They can be constructed using [`Namespace::new`], which will validate that
/// the provided `String` contains only valid characters.
///
/// The inner data can be accessed using the `as_str` method. `Namespace` also derefs
/// to a `&str` for convenience.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Namespace(String);

impl Namespace {
    const fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    /// Create a new `Namespace`.
    ///
    /// This validates that the provided string contains only ASCII alphanumeric,
    /// underscore (`_`) or hyphen (`-`) characters.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if the provided string contains invalid characters.
    pub fn new(s: String) -> Result<Self> {
        if s.chars().any(|c| !Self::is_valid_char(c)) {
            let invalid = s
                .chars()
                .filter(|c| !Self::is_valid_char(*c))
                .dedup()
                .collect();
            Err(Error::InvalidNamespace { full: s, invalid })
        } else {
            Ok(Self(s))
        }
    }

    /// Access the inner namespace as a `&str`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for Namespace {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The path of a channel.
///
/// Paths must only contain ASCII alphanumeric characters or any of `_-=/.`.
/// They can be constructed using [`Path::new`], which will validate that
/// the provided `String` contains only valid characters.
///
/// The inner data can be accessed using the `as_str` method. `Path` also derefs
/// to a `&str` for convenience.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path(String);

impl Path {
    const fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '=' || c == '/' || c == '.'
    }

    /// Create a new `Path`.
    ///
    /// This validates that the provided string contains only ASCII alphanumeric,
    /// underscore (`_`), hyphen (`-`), equals (`=`), forward-slash (`/`), or
    /// dot (`.`) characters.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if the provided string contains invalid characters.
    pub fn new(s: String) -> Result<Self> {
        if s.chars().any(|c| !Self::is_valid_char(c)) {
            let invalid = s
                .chars()
                .filter(|c| !Self::is_valid_char(*c))
                .dedup()
                .collect();
            Err(Error::InvalidPath { full: s, invalid })
        } else {
            Ok(Self(s))
        }
    }

    /// Access the inner path as a `&str`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Path {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The identifier of a pub/sub channel in Grafana Live.
///
/// Channels are represented as `/` delimited strings containing their three components,
/// and can be parsed from such strings using [`str::parse`].
/// When included in the [`Metadata`][crate::data::Metadata] of a [`Frame`][crate::data::Frame],
/// channels are (de)serialized from this format.
///
/// ```rust
/// # use grafana_plugin_sdk::live::{Channel, Scope};
/// // Note that the 'path' can contain '/'s.
/// let channel: Channel = "plugin/my-cool-plugin/streams/custom-streaming-feature"
///     .parse()
///     .expect("valid channel");
/// assert_eq!(channel.scope(), Scope::Plugin);
/// assert_eq!(channel.namespace().as_str(), "my-cool-plugin");
/// assert_eq!(channel.path().as_str(), "streams/custom-streaming-feature");
/// assert_eq!(
///     channel.to_string(),
///     String::from("plugin/my-cool-plugin/streams/custom-streaming-feature"),
/// );
/// ```
///
/// See the [channel guide] for more information.
///
/// [channel guide]: https://grafana.com/docs/grafana/latest/live/live-channel/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    scope: Scope,
    namespace: Namespace,
    path: Path,
}

impl Channel {
    /// Create a new channel from pre-validated parts.
    pub fn new(scope: Scope, namespace: Namespace, path: Path) -> Self {
        Self {
            scope,
            namespace,
            path,
        }
    }

    /// Get the scope of this channel.
    ///
    /// The scope determines the purpose of the channel; for example, channels used
    /// internally by Grafana have scope [`Grafana`][`Scope::Grafana`], while channels
    /// used by datasource plugins have scope [`Datasource`][`Scope::Datasource`].
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// Get the namespace of this channel.
    ///
    /// The namespace has a different meaning depending on scope:
    /// - when scope is [`Scope::Grafana`], namespace is a "feature"
    /// - when scope is [`Scope::Plugin`], namespace is the plugin name
    /// - when scope is [`Scope::Datasource`], namespace is the datasource `uid`.
    /// - when scope is [`Scope::Stream`], namespace is the stream ID.
    ///
    /// For example, scope [`Grafana`][`Scope::Grafana`] could have a namespace called `dashboard`,
    /// and all messages on such a channel would related to real-time dashboard events.
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Get the path of this channel.
    ///
    /// The path usually contains the identifier of some concrete resource within a namespace,
    /// such as the ID of a dashboard that a user is currently looking at.
    ///
    /// This can be anything the plugin author desires, provided it only includes the characters
    /// defined in the [`Path`] documentation.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl FromStr for Channel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::Empty);
        } else if s.len() > MAX_CHANNEL_LENGTH {
            return Err(Error::ExceedsMaxLength);
        }
        let mut parts = s.splitn(3, '/');
        let scope = parts
            .next()
            .ok_or(Error::MissingScope)
            .and_then(|x| x.parse())?;
        let namespace = parts
            .next()
            .ok_or(Error::MissingNamespace)
            .and_then(|x| Namespace::new(x.to_string()))?;
        let path = parts
            .next()
            .ok_or(Error::MissingPath)
            .and_then(|x| Path::new(x.to_string()))?;
        Ok(Self {
            scope,
            namespace,
            path,
        })
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}/{}", self.scope, self.namespace.0, self.path.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn success() {
        let chan: Channel = "grafana/dashboard/1234".parse().expect("valid channel");
        assert_eq!(chan.scope(), Scope::Grafana);
        assert_eq!(chan.namespace().as_str(), "dashboard");
        assert_eq!(chan.path().as_str(), "1234");
    }

    #[test]
    fn empty() {
        assert!("".parse::<Channel>().is_err())
    }

    #[test]
    fn exceeds_max_length() {
        let s: String = "grafana/dashboard/"
            .chars()
            .chain(std::iter::repeat('a').take(160))
            .collect();
        assert!(s.parse::<Channel>().is_err())
    }

    #[test]
    fn parse_valid() {
        assert!("Stream/cpu/test".parse::<Channel>().is_ok());
    }

    #[test]
    fn parse_valid_long_path() {
        assert!("stream/cpu/test".parse::<Channel>().is_ok());
    }

    #[test]
    fn parse_invalid_empty() {
        assert!("".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_path_empty() {
        assert!("stream/test".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_reserved_symbol() {
        assert!("stream/test/%".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_has_space() {
        assert!("stream/cpu/ test".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_has_unicode() {
        assert!("stream/cpu/ѓ".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_no_path() {
        assert!("grafana/bbb".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_invalid_only_scope() {
        assert!("grafana".parse::<Channel>().is_err());
    }

    #[test]
    fn parse_path_with_additional_symbols() {
        assert!("grafana/test/path/dash-and-equal=1.1.1.1"
            .parse::<Channel>()
            .is_ok());
    }

    #[test]
    fn parse_scope_namespace_with_additional_symbols() {
        assert!("grafana=/test=/path/dash-and-equal"
            .parse::<Channel>()
            .is_err());
    }

    #[test]
    fn display() {
        let chan: Channel = "stream/cpu/test".parse().expect("valid channel");
        assert_eq!(chan.to_string(), "stream/cpu/test");
    }
}
