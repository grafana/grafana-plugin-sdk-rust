use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, skip_serializing_none};

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

/// A structured, two-dimensional data frame.
///
/// The fields of this struct are public so it can be created manually if desired.
/// Alternatively, the [`IntoFrame`] trait provides a convenient way to create a
/// frame from an iterator of [`Field`]s.
///
/// # Examples
///
/// Creating a Frame directly:
///
/// ```rust
/// use grafana_plugin_sdk::{
///     data::{Field, Frame},
///     prelude::*,
/// };
///
/// let field = [1_u32, 2, 3].into_field("x");
///
/// let frame = Frame {
///     name: "direct".to_string(),
///     fields: vec![field],
///     meta: None,
///     ref_id: None,
/// };
/// ```
///
/// Using the constructor methods:
///
/// ```rust
/// use grafana_plugin_sdk::{
///     data::{Field, Frame},
///     prelude::*,
/// };
///
/// let field = [1_u32, 2, 3].into_field("x");
///
/// let mut new_frame = Frame::new("new".to_string());
/// new_frame.add_field(field);
/// let new_frame_with_metadata = Frame::new_with_metadata("new with metadata", Default::default());
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
pub struct Frame {
    /// The name of this frame.
    pub name: String,

    /// The fields of this frame.
    ///
    /// The data in all fields must be of the same length, but may have different types.
    pub fields: Vec<Field>,

    /// Optional metadata describing this frame.
    ///
    /// This can include custom metadata.
    pub meta: Option<Metadata>,

    /// The ID of this frame, used by the Grafana frontend.
    ///
    /// This does not usually need to be provided; it will be filled
    /// in by the SDK when the Frame is returned to Grafana.
    pub ref_id: Option<String>,
}

impl Frame {
    /// Create a new, empty `Frame` with no fields and no metadata.
    #[must_use]
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            meta: None,
            ref_id: None,
        }
    }

    /// Create a new, empty `Frame` with no fields and the given metadata.
    #[must_use]
    pub fn new_with_metadata<S: Into<String>>(name: S, metadata: Metadata) -> Self {
        Self {
            name: name.into(),
            fields: vec![],
            meta: Some(metadata),
            ref_id: None,
        }
    }

    /// Add a field to this frame.
    ///
    /// This method will not fail if the field's data has a different length to any existing fields,
    /// but this property will be checked later before the frame is serialized.
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
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

/// Convenience trait for converting iterators of [`Field`]s into a [`Frame`].
#[cfg_attr(docsrs, doc(notable_trait))]
pub trait IntoFrame {
    /// Create a [`Frame`] with the given name from `self`.
    fn into_frame<S: Into<String>>(self, name: S) -> Frame;
}

impl<T> IntoFrame for T
where
    T: IntoIterator<Item = Field>,
{
    fn into_frame<S: Into<String>>(self, name: S) -> Frame {
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
    fn from_fields<S: Into<String>>(name: S, fields: T) -> Frame;
}

impl<T: IntoFrame> FromFields<T> for Frame {
    fn from_fields<S: Into<String>>(name: S, fields: T) -> Frame {
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
