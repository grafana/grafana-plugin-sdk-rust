use std::marker::PhantomData;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none};
use thiserror::Error;

use crate::{
    data::{
        field::{Field, FieldConfig},
        ConfFloat64,
    },
    live::Channel,
};

pub(self) mod de;
pub(self) mod ser;
pub(crate) mod to_arrow;

use ser::{SerializableField, SerializableFrame, SerializableFrameData, SerializableFrameSchema};

/// The string representation for null values.
pub const EXPLICIT_NULL: &str = "null";
/// The standard name for time series time fields.
pub const TIME_FIELD_NAME: &str = "Time";
/// The standard name for time series value fields.
pub const VALUE_FIELD_NAME: &str = "Value";

mod sealed {
    pub trait Sealed {}
}

/// Marker trait used as a trait bound for the `Validity` type parameter of a [`Frame`].
pub trait FrameValidity: std::fmt::Debug + sealed::Sealed {}

/// Marker struct used to indicate that a `Frame` has been checked to be valid.
#[derive(Debug)]
pub struct Checked {
    _priv: (),
}
impl sealed::Sealed for Checked {}
impl FrameValidity for Checked {}

/// Marker struct used to indicate that a `Frame` has not been checked to be valid.
///
/// Unchecked `Frame`s can be checked and converted to checked frames using
/// [`Frame::check`].
#[derive(Debug)]
pub struct Unchecked {
    _priv: (),
}
impl sealed::Sealed for Unchecked {}
impl FrameValidity for Unchecked {}

/// An error occurring when handling a `Frame`.
#[derive(Debug, Error)]
pub enum FrameError {
    /// Occurs when a frame had mismatched field lengths while checking.
    #[error(
        "Mismatched frame field lengths: {}",
        frame.fields.iter().map(|x| format!("{} ({})", x.name, x.values.len())).join(", ")
    )]
    MismatchedFieldLengths {
        /// The original `Frame`, which can be fixed and reused if desired.
        frame: Frame<Unchecked>,
    },
}

/// A structured, two-dimensional data frame.
///
/// `Frame`s can be created manually using [`Frame::new`] if desired.
/// Alternatively, the [`IntoFrame`] and [`IntoCheckedFrame`] traits provide
/// a convenient way to create a frame from an iterator of [`Field`]s.
///
/// The type parameter `V` indicates whether the `Frame` has been confirmed to
/// be in a valid state. Frames begin in the `Checked` state when created by
/// [`Frame::new`], [`IntoCheckedFrame`], or by deserializing;
/// and transition to `Unchecked` when any `Field`s are added or modified.
/// They then remain in that state until [`Frame::check`] is called. This will
/// consume the `Frame`, validates that all of the fields have the correct length,
/// and returns a `Frame<Checked>` if successful. Checked frames can then be used
/// throughout the rest of the SDK (e.g. to be sent back to Grafana).
///
/// The [`IntoCheckedFrame::into_checked_frame`] method exists as a more direct
/// alternative to `iter.into_frame().check()`.
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
/// .into_frame("convenient");
/// ```
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Frame<V: FrameValidity> {
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

    phantom: PhantomData<V>,
}

// Impls only valid for checked frames.
impl Frame<Checked> {
    /// Create a new, empty `Frame` with no fields and no metadata.
    ///
    /// The returned frame is `Checked` because it begins with zero fields, which is
    /// always valid.
    ///
    /// # Examples
    ///
    /// Creating a Frame using the [`Frame::new`]:
    ///
    /// ```rust
    /// use grafana_plugin_sdk::{
    ///     data::{Checked, Frame},
    ///     prelude::*,
    /// };
    ///
    /// let frame: Frame<Checked> = Frame::new("frame");
    /// assert_eq!(frame.name, "frame".to_string());
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            meta: None,
            ref_id: None,
            phantom: PhantomData,
        }
    }

    /// Consumes this checked `Frame` and returns an unchecked frame.
    ///
    /// The returned unchecked `Frame` provides methods for accessing
    /// the `Fields` of the frame directly.
    #[must_use]
    pub fn into_unchecked(self) -> Frame<Unchecked> {
        Frame {
            name: self.name,
            fields: self.fields,
            meta: self.meta,
            ref_id: self.ref_id,
            phantom: PhantomData,
        }
    }

    /// Serialize this frame to JSON, returning the serialized bytes.
    ///
    /// Unlike the `Serialize` impl, this method allows for customization of the fields included.
    pub(crate) fn to_json(&self, include: FrameInclude) -> Result<Vec<u8>, serde_json::Error> {
        let schema_fields: Vec<_> = self
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
                    name: &self.name,
                    ref_id: self.ref_id.as_deref(),
                    meta: &self.meta,
                    fields: &schema_fields,
                }
            }),
            data: matches!(include, FrameInclude::All | FrameInclude::DataOnly).then(|| {
                SerializableFrameData {
                    fields: &self.fields,
                }
            }),
        };
        serde_json::to_vec(&ser)
    }
}

// Impls only valid for unchecked frames.
// In general this is anything that could manipulate `fields`.
impl Frame<Unchecked> {
    /// Add a field to this frame.
    ///
    /// This is similar to [`Frame::with_field`] but takes the frame by mutable reference,
    /// and is only available on unchecked frames.
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

    /// Get a mutable reference to the the `Fields` of this `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use arrow2::array::{PrimitiveArray, Utf8Array};
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
    ///         .downcast_ref::<PrimitiveArray<u32>>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<_>>(),
    ///     vec![Some(&4), Some(&5), Some(&6)],
    /// );
    /// assert_eq!(
    ///     frame
    ///         .fields()[1]
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<Utf8Array<i32>>()
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

    /// Check this unchecked Frame, returning a checked Frame ready for serializing.
    ///
    /// Frames begin in the `Unchecked` state when created by
    /// [`Frame::new`] or [`IntoFrame`], and
    /// remain in that state until [`Frame::check`] is called. This consumes the
    /// `Frame`, validates that all of the fields have the correct length, and
    /// returns a `Frame<Checked>` if successful. Checked frames can then be used
    /// throughout the rest of the SDK (e.g. to be sent back to Grafana).
    ///
    /// If the check fails this returns a [`FrameError`] which contains the original
    /// consumed `Frame`.
    ///
    /// # Errors
    ///
    /// Returns an error if the fields of `self` do not all have the same length.
    ///
    /// # Example
    ///
    /// ```rust
    /// use arrow2::array::{PrimitiveArray, Utf8Array};
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
    pub fn check(self) -> Result<Frame<Checked>, FrameError> {
        if self.fields.iter().map(|x| x.values.len()).all_equal() {
            Ok(Frame {
                name: self.name,
                fields: self.fields,
                meta: self.meta,
                ref_id: self.ref_id,
                phantom: PhantomData,
            })
        } else {
            Err(FrameError::MismatchedFieldLengths { frame: self })
        }
    }
}

// Impls valid for checked and unchecked frames.
// This is anything that doesn't touch `fields`; or, if the method
// _does_ touch `fields`, it should consume `self` and return an unchecked frame.
impl<V: FrameValidity> Frame<V> {
    /// Return a new frame with the given name.
    ///
    /// This consumes the input frame and returns a new `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::data::Frame;
    ///
    /// let frame = Frame::new("frame")
    ///     .with_name("other name");
    /// assert_eq!(frame.name, "other name".to_string());
    /// ```
    pub fn with_name(self, name: impl Into<String>) -> Frame<V> {
        Self {
            name: name.into(),
            fields: self.fields,
            meta: self.meta,
            ref_id: self.ref_id,
            phantom: PhantomData,
        }
    }

    /// Return a new frame with the given metadata.
    ///
    /// This consumes the input frame and returns a new `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::data::{Frame, Metadata};
    ///
    /// let frame = Frame::new("frame")
    ///     .with_metadata(None);
    /// assert_eq!(frame.meta, None);
    ///
    /// let frame = Frame::new("frame")
    ///     .with_metadata(Metadata {
    ///         path: Some("a path".to_string()),
    ///         ..Metadata::default()
    ///     });
    /// assert_eq!(frame.meta.unwrap().path, Some("a path".to_string()));
    /// ```
    #[must_use]
    pub fn with_metadata(self, metadata: impl Into<Option<Metadata>>) -> Frame<V> {
        Self {
            name: self.name,
            fields: self.fields,
            meta: metadata.into(),
            ref_id: self.ref_id,
            phantom: PhantomData,
        }
    }

    /// Return a new frame with an added field.
    ///
    /// This consumes the input frame and returns an unchecked `Frame` in its place.
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
    pub fn with_field(mut self, field: Field) -> Frame<Unchecked> {
        self.fields.push(field);
        Frame {
            name: self.name,
            fields: self.fields,
            meta: self.meta,
            ref_id: self.ref_id,
            phantom: PhantomData,
        }
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

/// Convenience trait for converting iterators of [`Field`]s into a [`Frame<Unchecked>`].
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait IntoFrame {
    /// Create a [`Frame<Unchecked>`] with the given name from `self`.
    fn into_frame(self, name: impl Into<String>) -> Frame<Unchecked>;
}

impl<T> IntoFrame for T
where
    T: IntoIterator<Item = Field>,
{
    fn into_frame(self, name: impl Into<String>) -> Frame<Unchecked> {
        Frame {
            name: name.into(),
            fields: self.into_iter().collect(),
            meta: None,
            ref_id: None,
            phantom: PhantomData,
        }
    }
}

/// Convenience trait for fallibly converting iterators of [`Field`]s into a [`Frame<Unchecked>`].
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait IntoCheckedFrame {
    /// Fallibly create a [`Frame<Checked>`] with the given name from `self`.
    ///
    /// This is the same as calling `IntoFrame::into_frame(iter, "name").check()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the fields of `self` do not all have the same length.
    fn into_checked_frame(self, name: impl Into<String>) -> Result<Frame<Checked>, FrameError>;
}

impl<T> IntoCheckedFrame for T
where
    T: IntoIterator<Item = Field>,
{
    fn into_checked_frame(self, name: impl Into<String>) -> Result<Frame<Checked>, FrameError> {
        Frame {
            name: name.into(),
            fields: self.into_iter().collect(),
            meta: None,
            ref_id: None,
            phantom: PhantomData,
        }
        .check()
    }
}

/// Convenience trait for creating a [`Frame<Unchecked>`] from an iterator of [`Field`]s.
///
/// This is the inverse of [`IntoFrame`] and is defined for all implementors of that trait.
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait FromFields<T: IntoFrame> {
    /// Create a [`Frame`] with the given name from `fields`.
    fn from_fields(name: impl Into<String>, fields: T) -> Frame<Unchecked>;
}

impl<T: IntoFrame> FromFields<T> for Frame<Unchecked> {
    fn from_fields(name: impl Into<String>, fields: T) -> Frame<Unchecked> {
        fields.into_frame(name)
    }
}

/// Options for customizing the way a [`Frame`] is serialized.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
}

/// A notification to be displayed in Grafana's UI.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// The severity level of a notice.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    /// Informational severity.
    Info,
    /// Warning severity.
    Warning,
    /// Error severity.
    Error,
}

/// A suggestion for which tab to display in the panel inspector in Grafana's UI.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
/// The `field_config` field's `name` property MUST be set.
///
/// This corresponds to the [`QueryResultMetaStat` on the frontend](https://github.com/grafana/grafana/blob/master/packages/grafana-data/src/types/data.ts#L53).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryStat {
    #[serde(flatten)]
    /// Information used to identify the field to which this statistic applies.
    pub field_config: FieldConfig,
    /// The actual statistic.
    pub value: ConfFloat64,
}
