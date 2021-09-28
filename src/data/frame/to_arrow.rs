use std::{collections::HashMap, sync::Arc};

use arrow2::{datatypes::Schema, io::ipc::write::FileWriter, record_batch::RecordBatch};
use thiserror::Error;

use crate::data::{field::Field, frame::Frame};

/// Errors occurring when serializing
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error serializing metadata")]
    Json(#[from] serde_json::Error),
    #[error("Error creating record batch")]
    CreateRecordBatch(arrow2::error::ArrowError),
    #[error("Error writing data to Arrow buffer")]
    WriteBuffer(arrow2::error::ArrowError),
}

impl Frame {
    /// Create an Arrow [`Schema`] for this Frame.
    ///
    /// If `ref_id` is provided, it is passed down to the various conversion
    /// function and takes precedence over the `ref_id` set on the frame.
    fn arrow_schema(&self, ref_id: Option<String>) -> Result<Schema, serde_json::Error> {
        let fields: Vec<_> = self
            .fields
            .iter()
            .map(Field::to_arrow_field)
            .collect::<Result<_, _>>()?;
        let mut metadata: HashMap<String, String> = IntoIterator::into_iter([
            ("name".to_string(), self.name.clone()),
            (
                "refId".to_string(),
                ref_id.unwrap_or_else(|| self.ref_id.clone().unwrap_or_default()),
            ),
        ])
        .collect();
        if let Some(meta) = &self.meta {
            metadata.insert("meta".to_string(), serde_json::to_string(&meta)?);
        }
        Ok(Schema::new_from(fields, metadata))
    }

    /// Convert this [`Frame`] to Arrow using the IPC format.
    ///
    /// If `ref_id` is provided, it is passed down to the various conversion
    /// function and takes precedence over the `ref_id` set on the frame.
    pub(crate) fn to_arrow(&self, ref_id: Option<String>) -> Result<Vec<u8>, Error> {
        let schema: Arc<Schema> = Arc::new(self.arrow_schema(ref_id)?);
        let records = RecordBatch::try_new(
            Arc::clone(&schema),
            self.fields.iter().map(|f| Arc::clone(&f.values)).collect(),
        )
        .map_err(Error::CreateRecordBatch)?;

        let mut buf = Vec::new();
        {
            let mut writer = FileWriter::try_new(&mut buf, &schema).map_err(Error::WriteBuffer)?;
            writer.write(&records).map_err(Error::WriteBuffer)?;
            writer.finish().map_err(Error::WriteBuffer)?;
        }
        Ok(buf)
    }
}