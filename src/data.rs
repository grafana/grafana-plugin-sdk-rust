use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Frame {
	name: String,
	fields: Vec<Field>,
	ref_id: Option<String>,
	meta: Option<Metadata>,
}

// impl Frame {
// 	fn new(name: String, field_length: usize, field_types: &[FieldType]) -> Self {
// 		Self {
// 			name,
// 			fields: field_types.iter().map(|ft| Field::new(ft, field_length)),
// 			ref_id: None,
// 			meta: None,
// 		}
// 	}
// }

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
	path: Option<String>,

	path_separator: Option<String>,

	custom: Option<Map<String, Value>>,

	stats: Option<Vec<QueryStat>>,

	notices: Option<Vec<Notice>>,

	preferred_visualisation: Option<VisType>,

	executed_query_string: Option<String>,
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum VisType {
	Graph,
	Table,
	Logs,
	Trace,
	NodeGraph,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Field {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Notice {
	severity: NoticeSeverity,
	text: String,
	link: Option<String>,
	inspect: Option<InspectType>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum NoticeSeverity {
	Info,
	Warning,
	Error,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum InspectType {
	None,
	Meta,
	Error,
	Data,
	Stats,
}

fn serialize_conf_float64<S: Serializer>(val: &Option<f64>, s: S) -> Result<S::Ok, S::Error> {
	if let Some(f) = val {
		if f.is_nan() || f.is_infinite() {
			s.serialize_none()
		} else {
			s.serialize_f64(*f)
		}
	} else {
		s.serialize_none()
	}
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct ConfFloat64(#[serde(serialize_with = "serialize_conf_float64")] Option<f64>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryStat {
	field_config: FieldConfig,
	value: ConfFloat64,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FieldConfig {
	display_name: Option<String>,

	#[serde(rename = "displayNameFromDS")]
	display_name_from_ds: Option<String>,

	path: Option<String>,

	description: Option<String>,

	filterable: Option<bool>,

	unit: Option<String>,
	decimals: Option<u16>,
	min: Option<f64>,
	max:  Option<f64>,

	mappings: Vec<ValueMapping>,
	thresholds: ThresholdsConfig,

	links: Vec<DataLink>,

	no_value: String,

	custom: HashMap<String, Value>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValueMapping {
	id: Option<i16>,
	text: Option<String>,
	_type: ValueMappingData,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ValueMappingData {
	ValueToText { value: Option<String> },
	RangeToText { from: Option<String>, to: Option<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThresholdsConfig {
	mode: ThresholdsMode,
	steps: Vec<Threshold>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Threshold {
	value: Option<ConfFloat64>,
	color: Option<String>,
	state: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ThresholdsMode {
	Absolute,
	Percentage,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataLink {
	title: Option<String>,
	target_blank: Option<bool>,
	url: Option<String>,
}