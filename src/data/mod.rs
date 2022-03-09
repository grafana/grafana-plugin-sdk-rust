//! Data types used throughout the SDK.
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
