//! Data types used throughout the SDK.
//!
//! Note that many of the types in this module are marked as `#[non_exhaustive]` and cannot
//! be constructed using struct expressions. This is because Grafana may choose to add
//! additional fields or variants at any time. Instead, use the constructor (if available) or
//! create a mutable default value using `Default::default()` and modify any fields.
//!
//! For example:
//!
//! ```
//! use grafana_plugin_sdk::{
//!     data::{Field, FieldConfig, Frame, Metadata, Notice, Severity},
//!     prelude::*,
//! };
//!
//! let mut field_config = FieldConfig::default();
//! field_config.display_name_from_ds = Some("X".to_string());
//! let field = ["a", "b", "c"]
//!     .into_field("x")
//!     .with_config(field_config);
//!
//! let mut notice = Notice::new("warning! this is important".to_string());
//! notice.severity = Some(Severity::Warning);
//! let mut metadata = Metadata::default();
//! metadata.path = Some("my path".to_string());
//! metadata.notices = Some(vec![notice]);
//!
//! let frame = Frame::new("my frame")
//!     .with_field(field)
//!     .with_metadata(metadata);
//! # assert_eq!(frame.fields()[0].config.as_ref().unwrap().display_name_from_ds, Some("X".to_string()));
//! ```
use serde::{ser::Serializer, Deserialize, Serialize};

mod error;
mod field;
mod field_type;
mod frame;

pub use error::Error;
pub use field::*;
pub use field_type::{FieldType, IntoFieldType};
pub use frame::*;

/// A wrapper around an `Option<f64>` used in various backend data structures, with custom NaN and Infinity serialization.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfFloat64(#[serde(serialize_with = "serialize_conf_float64")] Option<f64>);

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
