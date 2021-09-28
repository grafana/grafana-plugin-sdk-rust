use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::data::{
    field::{Field, FieldConfig},
    ConfFloat64,
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
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Frame {
    /// The name of this [`Frame`].
    pub name: String,
    /// The fields of this [`Frame`].
    ///
    /// The data in all fields must be of the same length, but may have different types.
    pub fields: Vec<Field>,
    /// Optional metadata describing this [`Frame`].
    pub meta: Option<Metadata>,
    /// The ID of this [`Frame`], used by the Grafana frontend.
    ///
    /// This does not usually need to be provided; it will be filled
    /// in by the SDK when the Frame is returned to Grafana.
    pub ref_id: Option<String>,
}

impl Frame {
    /// Create a new, empty `Frame` with no fields and no metadata.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self {
            name,
            fields: vec![],
            meta: None,
            ref_id: None,
        }
    }

    /// Create a new, empty `Frame` with no fields and the given metadata.
    #[must_use]
    pub const fn new_with_metadata(name: String, metadata: Metadata) -> Self {
        Self {
            name,
            fields: vec![],
            meta: Some(metadata),
            ref_id: None,
        }
    }

    /// Add a field to this frame.
    ///
    /// This will not fail if the field's data has a different length to any existing fields,
    /// but this will be checked later before the frame is serialized.
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
                name: f.name.as_deref(),
                labels: f.labels.as_ref(),
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
    pub fn set_channel(&mut self, channel: String) {
        self.meta = Some(std::mem::take(&mut self.meta).unwrap_or_default());
        self.meta.as_mut().unwrap().channel = Some(channel);
    }
}

pub trait IntoFrame {
    fn into_frame(self, name: String) -> Frame;
}

impl<T> IntoFrame for T
where
    T: IntoIterator<Item = Field>,
{
    fn into_frame(self, name: String) -> Frame {
        Frame {
            name,
            fields: self.into_iter().collect(),
            meta: None,
            ref_id: None,
        }
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

#[skip_serializing_none]
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
    #[serde(default)]
    pub channel: Option<String>,

    /// The preferred visualisation option when used in Grafana's Explore mode.
    #[serde(rename = "preferredVisualisationType", default)]
    pub preferred_visualisation: Option<VisType>,

    /// The raw query sent to the underlying system after all macros and templating have been applied.
    ///
    /// This will be shown in the query inspector if present.
    #[serde(default)]
    pub executed_query_string: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Visualiation type options understood by Grafana.
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

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    #[serde(default)]
    pub severity: Option<NoticeSeverity>,
    pub text: String,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub inspect: Option<InspectType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum NoticeSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum InspectType {
    None,
    Meta,
    Error,
    Data,
    Stats,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryStat {
    #[serde(flatten)]
    pub field_config: FieldConfig,
    pub value: ConfFloat64,
}
