use thiserror::Error;

use super::frame::to_arrow;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Arrow serialization error")]
    ArrowSerialization(#[from] to_arrow::Error),
    #[error("Data type mismatch")]
    DataTypeMismatch,
}
