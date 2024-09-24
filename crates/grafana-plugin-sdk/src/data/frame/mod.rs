use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none};

use crate::{
    data::{
        field::{Field, FieldConfig},
        ConfFloat64, Error,
    },
    live::Channel,
};

mod de;
mod ser;
pub(crate) mod to_arrow;

use ser::{SerializableField, SerializableFrame, SerializableFrameData, SerializableFrameSchema};

/// The string representation for null values.
pub const EXPLICIT_NULL: &str = "null";
/// The standard name for time series time fields.
pub const TIME_FIELD_NAME: &str = "Time";
/// The standard name for time series value fields.
pub const VALUE_FIELD_NAME: &str = "Value";

/// A structured, two-dimensional data frame.
///
/// `Frame`s can be created manually using [`Frame::new`] if desired.
/// Alternatively, the [`IntoFrame`] trait (and its reciprocal, [`FromFields`])
/// provide a convenient way to create a frame from an iterator of [`Field`]s.
///
/// Frames generally can't be passed back to the SDK without first being checked
/// that they are valid. The [`Frame::check`] method performs all required checks
/// (e.g. that all fields have the same length), and returns a [`CheckedFrame<'_>`]
/// which contains a reference to the original frame. This can then be used
/// throughout the rest of the SDK (e.g. to be sent back to Grafana).
///
/// # Examples
///
/// Creating a Frame using the [`Frame::new`]:
///
/// ```rust
/// use grafana_plugin_sdk::{
///     data::{Field, Frame},
///     prelude::*,
/// };
///
/// let field = [1_u32, 2, 3].into_field("x");
/// let frame = Frame::new("new")
///     .with_field(field);
/// ```
///
/// Using the [`IntoFrame`] trait:
///
/// ```rust
/// use grafana_plugin_sdk::prelude::*;
///
/// let frame = [
///     [1_u32, 2, 3].into_field("x"),
///     ["a", "b", "c"].into_field("y"),
/// ]
/// .into_frame("super convenient");
/// ```
///
/// Fields can be accessed using either [`Frame::fields`] and [`Frame::fields_mut`], or
/// by using the [`Index`][std::ops::Index] and [`IndexMut`][std::ops::IndexMut] implementations
/// with field indexes or names:
///
/// ```rust
/// use grafana_plugin_sdk::prelude::*;
///
/// let frame = [
///     [1_u32, 2, 3].into_field("x"),
///     ["a", "b", "c"].into_field("y"),
/// ]
/// .into_frame("frame");
///
/// assert_eq!(
///     frame.fields()[0].name,
///     "x",
/// );
/// assert_eq!(
///     frame.fields()[1],
///     frame["y"],
/// );
/// ```
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Frame {
    /// The name of this frame.
    pub name: String,

    /// Optional metadata describing this frame.
    ///
    /// This can include custom metadata.
    pub meta: Option<Metadata>,

    /// The ID of this frame, used by the Grafana frontend.
    ///
    /// This does not usually need to be provided; it will be filled
    /// in by the SDK when the Frame is returned to Grafana.
    pub(crate) ref_id: Option<String>,

    /// The fields of this frame.
    ///
    /// The data in all fields must be of the same length, but may have different types.
    fields: Vec<Field>,
}

impl Frame {
    /// Create a new, empty `Frame` with no fields and no metadata.
    ///
    /// # Examples
    ///
    /// Creating a Frame using the [`Frame::new`]:
    ///
    /// ```rust
    /// use grafana_plugin_sdk::{
    ///     data::Frame,
    ///     prelude::*,
    /// };
    ///
    /// let frame = Frame::new("frame");
    /// assert_eq!(&frame.name, "frame");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            meta: None,
            ref_id: None,
        }
    }

    /// Add a field to this frame.
    ///
    /// This is similar to [`Frame::with_field`] but takes the frame by mutable reference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut frame = [
    ///     [1_u32, 2, 3].into_field("x"),
    /// ]
    /// .into_frame("frame");
    /// assert_eq!(frame.fields().len(), 1);
    /// frame.add_field(["a", "b", "c"].into_field("y"));
    /// assert_eq!(frame.fields().len(), 2);
    /// assert_eq!(&frame.fields()[1].name, "y");
    /// ```
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Get an immutable reference to the the `Fields` of this `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut frame = [
    ///     [1_u32, 2, 3].into_field("x"),
    /// ]
    /// .into_frame("frame");
    /// assert_eq!(frame.fields().len(), 1);
    /// assert_eq!(&frame.fields()[0].name, "x");
    /// ```
    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Get a mutable reference to the the `Fields` of this `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use arrow::{array::{PrimitiveArray, StringArray}, datatypes::UInt32Type};
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// // Create an initial `Frame`.
    /// let mut frame = [
    ///     [1_u32, 2, 3].into_field("x"),
    ///     ["a", "b", "c"].into_field("y"),
    /// ]
    /// .into_frame("frame");
    ///
    /// // Update the frame's fields.
    /// frame.fields_mut()[0].set_values(4u32..7).unwrap();
    /// frame.fields_mut()[1].set_values(["d", "e", "f"]).unwrap();
    ///
    /// assert_eq!(
    ///     frame
    ///         .fields()[0]
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<PrimitiveArray<UInt32Type>>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<Option<u32>>>(),
    ///     vec![Some(4), Some(5), Some(6)],
    /// );
    /// assert_eq!(
    ///     frame
    ///         .fields()[1]
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<StringArray>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<_>>(),
    ///     vec![Some("d"), Some("e"), Some("f")],
    /// );
    /// assert!(frame.check().is_ok());
    /// ```
    pub fn fields_mut(&mut self) -> &mut [Field] {
        &mut self.fields
    }

    /// Check this unchecked Frame, returning a [`CheckedFrame`] ready for serializing.
    ///
    /// `Frame`s may be in an intermediate state whilst being constructed (for example,
    /// their field lengths may differ). Calling this method validates the frame and
    /// returns a `CheckedFrame` if successful. Checked frames can then be used
    /// throughout the rest of the SDK (e.g. to be sent back to Grafana).
    ///
    /// # Errors
    ///
    /// Returns an error if the fields of `self` do not all have the same length.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// assert!(
    ///     [
    ///         [1_u32, 2, 3].into_field("x"),
    ///         ["a", "b", "c"].into_field("y"),
    ///     ]
    ///     .into_frame("frame")
    ///     .check()
    ///     .is_ok()
    /// );
    ///
    /// assert!(
    ///     [
    ///         [1_u32, 2, 3, 4].into_field("x"),
    ///         ["a", "b", "c"].into_field("y"),
    ///     ]
    ///     .into_frame("frame")
    ///     .check()
    ///     .is_err()
    /// );
    /// ```
    pub fn check(&self) -> Result<CheckedFrame<'_>, Error> {
        if self.fields.iter().map(|x| x.values.len()).all_equal() {
            Ok(CheckedFrame(self))
        } else {
            Err(Error::FieldLengthMismatch {
                lengths: self
                    .fields
                    .iter()
                    .map(|x| (x.name.to_string(), x.values.len()))
                    .collect(),
            })
        }
    }

    /// Return a new frame with the given name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::data::Frame;
    ///
    /// let frame = Frame::new("frame")
    ///     .with_name("other name");
    /// assert_eq!(&frame.name, "other name");
    /// ```
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Return a new frame with the given metadata.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::data::{Frame, Metadata, Notice, Severity};
    ///
    /// let mut notice = Notice::new("read this notice".to_string());
    /// notice.severity = Some(Severity::Warning);
    /// let mut metadata = Metadata::default();
    /// metadata.path = Some("a path".to_string());
    /// metadata.notices = Some(vec![notice]);
    /// let frame = Frame::new("frame").with_metadata(metadata);
    /// assert_eq!(frame.meta.unwrap().path, Some("a path".to_string()));
    /// ```
    #[must_use]
    pub fn with_metadata(mut self, metadata: impl Into<Option<Metadata>>) -> Self {
        self.meta = metadata.into();
        self
    }

    /// Return a new frame with an added field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::{data::Frame, prelude::*};
    ///
    /// let frame = Frame::new("frame")
    ///     .with_field([1_u32, 2, 3].into_field("x"))
    ///     .with_field(["a", "b", "c"].into_field("y"));
    /// assert_eq!(frame.fields().len(), 2);
    /// assert_eq!(&frame.fields()[0].name, "x");
    /// assert_eq!(&frame.fields()[1].name, "y");
    /// ```
    #[must_use]
    pub fn with_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Return a new frame with added fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::{data::Frame, prelude::*};
    ///
    /// let frame = Frame::new("frame")
    ///     .with_fields([
    ///         [1_u32, 2, 3].into_field("x"),
    ///         ["a", "b", "c"].into_field("y"),
    ///     ]);
    /// assert_eq!(frame.fields().len(), 2);
    /// assert_eq!(&frame.fields()[0].name, "x");
    /// assert_eq!(&frame.fields()[1].name, "y");
    /// ```
    #[must_use]
    pub fn with_fields(mut self, fields: impl IntoIterator<Item = Field>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Set the channel of the frame.
    ///
    /// This is used when a frame can be 'upgraded' to a streaming response, to
    /// tell Grafana that the given channel should be used to subscribe to updates
    /// to this frame.
    pub fn set_channel(&mut self, channel: Channel) {
        self.meta = Some(std::mem::take(&mut self.meta).unwrap_or_default());
        self.meta.as_mut().unwrap().channel = Some(channel);
    }
}

impl std::ops::Index<usize> for Frame {
    type Output = Field;
    fn index(&self, index: usize) -> &Self::Output {
        &self.fields()[index]
    }
}

impl std::ops::IndexMut<usize> for Frame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields_mut()[index]
    }
}

impl std::ops::Index<&str> for Frame {
    type Output = Field;
    fn index(&self, name: &str) -> &Self::Output {
        self.fields().iter().find(|x| x.name == name).unwrap()
    }
}

impl std::ops::IndexMut<&str> for Frame {
    fn index_mut(&mut self, name: &str) -> &mut Self::Output {
        self.fields_mut()
            .iter_mut()
            .find(|x| x.name == name)
            .unwrap()
    }
}

/// A reference to a checked and verified [`Frame`], which is ready for serialization.
pub struct CheckedFrame<'a>(&'a Frame);

impl CheckedFrame<'_> {
    /// Serialize this frame to JSON, returning the serialized bytes.
    ///
    /// Unlike the `Serialize` impl, this method allows for customization of the fields included.
    pub(crate) fn to_json(&self, include: FrameInclude) -> Result<Vec<u8>, serde_json::Error> {
        let schema_fields: Vec<_> = self
            .0
            .fields
            .iter()
            .map(|f| SerializableField {
                name: &f.name,
                labels: &f.labels,
                config: f.config.as_ref(),
                type_: f.type_info.simple_type(),
                type_info: &f.type_info,
            })
            .collect();
        let ser = SerializableFrame {
            schema: matches!(include, FrameInclude::All | FrameInclude::SchemaOnly).then(|| {
                SerializableFrameSchema {
                    name: &self.0.name,
                    ref_id: self.0.ref_id.as_deref(),
                    meta: &self.0.meta,
                    fields: &schema_fields,
                }
            }),
            data: matches!(include, FrameInclude::All | FrameInclude::DataOnly).then(|| {
                SerializableFrameData {
                    fields: &self.0.fields,
                }
            }),
        };
        serde_json::to_vec(&ser)
    }
}

/// Convenience trait for converting iterators of [`Field`]s into a [`Frame`].
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait IntoFrame {
    /// Create a [`Frame`] with the given name from `self`.
    fn into_frame(self, name: impl Into<String>) -> Frame;
}

impl<T> IntoFrame for T
where
    T: IntoIterator<Item = Field>,
{
    fn into_frame(self, name: impl Into<String>) -> Frame {
        Frame {
            name: name.into(),
            fields: self.into_iter().collect(),
            meta: None,
            ref_id: None,
        }
    }
}

/// Convenience trait for creating a [`Frame`] from an iterator of [`Field`]s.
///
/// This is the inverse of [`IntoFrame`] and is defined for all implementors of that trait.
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait FromFields<T: IntoFrame> {
    /// Create a [`Frame`] with the given name from `fields`.
    fn from_fields(name: impl Into<String>, fields: T) -> Frame;
}

impl<T: IntoFrame> FromFields<T> for Frame {
    fn from_fields(name: impl Into<String>, fields: T) -> Frame {
        fields.into_frame(name)
    }
}

/// Options for customizing the way a [`Frame`] is serialized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum FrameInclude {
    /// Serialize both the schema and data.
    All,
    /// Serialize just the data.
    DataOnly,
    /// Serialize just the schema.
    SchemaOnly,
}

/// Metadata about a [`Frame`].
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Metadata {
    /// A browsable path on the datasource.
    #[serde(default)]
    pub path: Option<String>,

    /// Defines the separator pattern to decode a hierarchy. The default separator is '/'.
    #[serde(default)]
    pub path_separator: Option<String>,

    /// Custom datasource specific values.
    #[serde(default)]
    pub custom: Option<Map<String, Value>>,

    /// Query result statistics.
    #[serde(default)]
    pub stats: Option<Vec<QueryStat>>,

    /// Additional information about the data in the frame that Grafana can display in the UI.
    #[serde(default)]
    pub notices: Option<Vec<Notice>>,

    /// Path to a stream in Grafana Live that has real-time updates for this data.
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    #[serde(default)]
    pub channel: Option<Channel>,

    /// The preferred visualisation option when used in Grafana's Explore mode.
    #[serde(rename = "preferredVisualisationType", default)]
    pub preferred_visualisation: Option<VisType>,

    /// The raw query sent to the underlying system after all macros and templating have been applied.
    ///
    /// This will be shown in the query inspector if present.
    #[serde(default)]
    pub executed_query_string: Option<String>,
}

/// Visualiation type options understood by Grafana.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum VisType {
    /// Graph visualisation.
    Graph,
    /// Table visualisation.
    Table,
    /// Logs visualisation.
    Logs,
    /// Trace visualisation.
    Trace,
    /// Node graph visualization.
    NodeGraph,
    /// Flame graph visualization.
    FlameGraph,
}

/// A notification to be displayed in Grafana's UI.
///
/// # Example
///
/// ```
/// use grafana_plugin_sdk::{data::{Frame, Metadata, Notice, Severity}, prelude::*};
///
/// let mut notice = Notice::new("Isn't this a cool set of data?".to_string());
/// notice.severity = Some(Severity::Info);
///
/// let mut metadata = Metadata::default();
/// metadata.notices = Some(vec![notice]);
///
/// let frame = Frame::new("frame")
///     .with_fields([
///         [1_u32, 2, 3].into_field("x"),
///         ["a", "b", "c"].into_field("y"),
///     ])
///     .with_metadata(metadata);
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Notice {
    /// The severity level of this notice.
    #[serde(default)]
    pub severity: Option<Severity>,

    /// Freeform descriptive text to display on the notice.
    pub text: String,

    /// An optional link to display in the Grafana UI.
    ///
    /// Can be an absolute URL or a path relative to Grafana's root URL.
    #[serde(default)]
    pub link: Option<String>,

    /// An optional suggestion for which tab to display in the panel inspector in Grafana's UI.
    #[serde(default)]
    pub inspect: Option<InspectType>,
}

impl Notice {
    /// Create a new `Notice` with the given text.
    ///
    /// ```
    /// use grafana_plugin_sdk::data::Notice;
    ///
    /// let notice = Notice::new("Isn't this a cool set of data?".to_string());
    /// ```
    pub fn new(text: String) -> Self {
        Self {
            text,
            severity: None,
            link: None,
            inspect: None,
        }
    }
}

/// The severity level of a notice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Severity {
    /// Informational severity.
    Info,
    /// Warning severity.
    Warning,
    /// Error severity.
    Error,
}

/// A suggestion for which tab to display in the panel inspector in Grafana's UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum InspectType {
    /// Do not suggest anything.
    None,
    /// Suggest the "meta" tab of the panel inspector.
    Meta,
    /// Suggest the "error" tab of the panel inspector.
    Error,
    /// Suggest the "data" tab of the panel inspector.
    Data,
    /// Suggest the "stats" tab of the panel inspector.
    Stats,
}

/// Used for storing arbitrary statistics metadata related to a query and its results.
///
/// Examples include total request time and data processing time.
///
/// The `field_config` field's `display_name` property MUST be set.
///
/// This corresponds to the [`QueryResultMetaStat` on the frontend](https://github.com/grafana/grafana/blob/master/packages/grafana-data/src/types/data.ts#L53).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryStat {
    #[serde(flatten)]
    /// Information used to identify the field to which this statistic applies.
    pub field_config: FieldConfig,
    /// The actual statistic.
    pub value: ConfFloat64,
}

impl QueryStat {
    /// Create a new `QueryStat ` for a field.
    pub fn new(field_config: FieldConfig, value: ConfFloat64) -> Self {
        Self {
            field_config,
            value,
        }
    }
}
