//! Error types returned by the SDK.
use arrow::datatypes::DataType;
use itertools::Itertools;
use thiserror::Error;

use super::frame::to_arrow;

/// Errors that can occur when interacting with the Grafana plugin SDK.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error has occurred when serializing to Arrow IPC format.
    #[error("Arrow serialization error: {0}")]
    ArrowSerialization(#[from] to_arrow::Error),

    /// There is a datatype mismatch in a field.
    ///
    /// This can happen when calling [`Field::set_values`][crate::data::Field::set_values] with an array whose datatype
    /// does not match the existing array.
    #[error(
        "Data type mismatch in field {} (existing: {existing:?}, new: {new:?})",
        field
    )]
    DataTypeMismatch {
        /// The existing datatype of the field.
        existing: DataType,
        /// The datatype of the new data.
        new: DataType,
        /// The name of the field.
        field: String,
    },

    /// Occurs when a frame had mismatched field lengths while checking.
    #[error(
        "Frame field length mismatch: {}",
        .lengths.iter().map(|x| format!("{} ({})", x.0, x.1)).join(", ")
    )]
    FieldLengthMismatch {
        /// The names and lengths of the fields in the `Frame`.
        lengths: Vec<(String, usize)>,
    },

    /// A field was created using an Arrow array with a datatype unsupported by Grafana.
    #[error("Unsupported Arrow data type: {0:?}")]
    UnsupportedArrowDataType(DataType),
}
