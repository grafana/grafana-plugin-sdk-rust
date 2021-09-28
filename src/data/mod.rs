use serde::{ser::Serializer, Deserialize, Serialize};

mod error;
mod field;
mod field_type;
mod frame;

pub use error::Error;
pub use field::*;
pub use frame::*;

pub mod prelude {
    pub use super::field::{ArrayIntoField, IntoField, IntoOptField};
    pub use super::frame::IntoFrame;
}

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
