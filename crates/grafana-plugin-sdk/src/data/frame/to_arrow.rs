//! Conversion of [`Frame`][crate::data::Frame]s to the Arrow IPC format.
use std::collections::BTreeMap;

use arrow2::{chunk::Chunk, datatypes::Schema, io::ipc::write::FileWriter};
use thiserror::Error;

use crate::data::{field::Field, frame::CheckedFrame};

/// Errors occurring when serializing a [`Frame`][crate::data::Frame] to the Arrow IPC format.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred converting the frame's metadata to JSON.
    #[error("Error serializing metadata")]
    Json(#[from] serde_json::Error),
    /// An error occurred creating the Arrow record batch.
    #[error("Error creating record batch: {0}")]
    CreateRecordBatch(arrow2::error::Error),
    /// An error occurred when attempting to create or write data to the output buffer.
    #[error("Error writing data to Arrow buffer")]
    WriteBuffer(arrow2::error::Error),
}

impl CheckedFrame<'_> {
    /// Create an Arrow [`Schema`] for this Frame.
    ///
    /// If `ref_id` is provided, it is passed down to the various conversion
    /// function and takes precedence over the `ref_id` set on the frame.
    fn arrow_schema(&self, ref_id: Option<String>) -> Result<Schema, serde_json::Error> {
        let fields: Vec<_> = self
            .0
            .fields
            .iter()
            .map(Field::to_arrow_field)
            .collect::<Result<_, _>>()?;
        let mut metadata: BTreeMap<String, String> = [
            ("name".to_string(), self.0.name.to_string()),
            (
                "refId".to_string(),
                ref_id.unwrap_or_else(|| {
                    self.0
                        .ref_id
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or_default()
                }),
            ),
        ]
        .into_iter()
        .collect();
        if let Some(meta) = &self.0.meta {
            metadata.insert("meta".to_string(), serde_json::to_string(&meta)?);
        }
        Ok(Schema::from(fields).with_metadata(metadata))
    }

    /// Convert this [`Frame`][crate::data::Frame] to Arrow using the IPC format.
    ///
    /// If `ref_id` is provided, it is passed down to the various conversion
    /// function and takes precedence over the `ref_id` set on the frame.
    pub(crate) fn to_arrow(&self, ref_id: Option<String>) -> Result<Vec<u8>, Error> {
        let schema = self.arrow_schema(ref_id)?;

        let records = if self.0.fields.is_empty() {
            None
        } else {
            Some(
                Chunk::try_new(self.0.fields.iter().map(|f| f.values.clone()).collect())
                    .map_err(Error::CreateRecordBatch)?,
            )
        };

        let mut buf = Vec::new();
        {
            let mut writer = FileWriter::try_new(&mut buf, schema, None, Default::default())
                .map_err(Error::WriteBuffer)?;
            if let Some(records) = records {
                writer.write(&records, None).map_err(Error::WriteBuffer)?;
            }
            writer.finish().map_err(Error::WriteBuffer)?;
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod test {
    use crate::data;

    #[test]
    fn can_convert_frame_with_empty_schema() {
        let frame = data::Frame::new("test");
        assert!(frame.check().unwrap().to_arrow(None).is_ok());
    }
}
