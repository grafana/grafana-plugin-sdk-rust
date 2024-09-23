//! Deserialization of [`Frame`]s from the JSON format.
use std::{collections::BTreeMap, fmt, marker::PhantomData, sync::Arc};

use arrow::{
    array::{
        Array, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array,
        Int8Array, StringArray, TimestampNanosecondArray, UInt16Array, UInt32Array, UInt64Array,
        UInt8Array,
    },
    datatypes::{
        ArrowPrimitiveType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
        UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
};
use num_traits::Float;
use serde::{
    de::{DeserializeOwned, DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor},
    Deserialize,
};

use crate::data::{
    field::{FieldConfig, SimpleType, TypeInfo, TypeInfoType},
    frame::ser::Entities,
    Frame, Metadata,
};

// Deserialization from the JSON representation for [`Frame`]s.
//
// This follows the [Manually deserializing a struct](https://serde.rs/deserialize-struct.html)
// example in the [`serde`] docs, and uses [`RawValue`] to avoid intermediate buffering
// of arrays during deserialization.
impl<'de> Deserialize<'de> for Frame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Schema,
            Data,
        }

        struct FrameVisitor;

        impl<'de> Visitor<'de> for FrameVisitor {
            type Value = Frame;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("stuct Frame")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut schema: Option<Schema> = None;
                let mut raw_data: Option<RawData> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Schema => {
                            if schema.is_some() {
                                return Err(Error::duplicate_field("schema"));
                            }
                            schema = Some(map.next_value()?);
                        }
                        Field::Data => {
                            if raw_data.is_some() {
                                return Err(Error::duplicate_field("data"));
                            }
                            raw_data = Some(map.next_value()?);
                        }
                    }
                }
                let schema = schema.ok_or_else(|| Error::missing_field("schema"))?;
                let raw_data = raw_data.ok_or_else(|| Error::missing_field("data"))?;
                let data: Data = (&schema, raw_data)
                    .try_into()
                    .map_err(|e| Error::custom(format!("invalid values: {}", e)))?;

                Ok(Frame {
                    name: schema.name,
                    ref_id: Some(schema.ref_id),
                    meta: schema.meta,
                    fields: schema
                        .fields
                        .into_iter()
                        .zip(data.values)
                        .map(|(f, values)| crate::data::field::Field {
                            name: f.name,
                            labels: f.labels,
                            config: f.config,
                            values,
                            type_info: f.type_info,
                        })
                        .collect(),
                })
            }
        }

        const FIELDS: &[&str] = &["schema", "data"];
        deserializer.deserialize_struct("Frame", FIELDS, FrameVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Field {
    #[serde(default)]
    name: String,
    #[serde(default)]
    labels: BTreeMap<String, String>,
    #[serde(default)]
    config: Option<FieldConfig>,
    #[serde(rename = "type")]
    _type: SimpleType,
    type_info: TypeInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Schema {
    name: String,
    ref_id: String,
    #[serde(default)]
    fields: Vec<Field>,
    #[serde(default)]
    meta: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct RawData<'a> {
    #[serde(borrow, default)]
    values: Vec<&'a serde_json::value::RawValue>,
    #[serde(default)]
    entities: Option<Vec<Option<Entities>>>,
}

#[derive(Debug)]
struct Data {
    values: Vec<Arc<dyn Array>>,
}

impl TryFrom<(&'_ Schema, RawData<'_>)> for Data {
    type Error = serde_json::Error;

    /// Create `Data` from the schema and data objects found in the JSON representation.
    ///
    /// This handles deserializing each of the arrays in `values` with the correct datatypes,
    /// replacing any 'entities' (`NaN`, `Inf`, `-Inf`) in those arrays, and converting any
    /// which require it (e.g. timestamps).
    fn try_from((schema, data): (&'_ Schema, RawData<'_>)) -> Result<Self, Self::Error> {
        // Handle entity replacement, return values.
        let fields = schema.fields.iter();
        let values = data.values.into_iter();
        let entities = data
            .entities
            .unwrap_or_else(|| vec![None; fields.len()])
            .into_iter();
        Ok(Self {
            values: fields
                .zip(values)
                .zip(entities)
                .map(|((f, v), e)| {
                    let s = v.get();
                    let arr: Arc<dyn Array> = match f.type_info.frame {
                        TypeInfoType::Int8 => parse_primitive_array::<Int8Array, Int8Type>(s)?,
                        TypeInfoType::Int16 => parse_primitive_array::<Int16Array, Int16Type>(s)?,
                        TypeInfoType::Int32 => parse_primitive_array::<Int32Array, Int32Type>(s)?,
                        TypeInfoType::Int64 => parse_primitive_array::<Int64Array, Int64Type>(s)?,
                        TypeInfoType::UInt8 => parse_primitive_array::<UInt8Array, UInt8Type>(s)?,
                        TypeInfoType::UInt16 => {
                            parse_primitive_array::<UInt16Array, UInt16Type>(s)?
                        }
                        TypeInfoType::UInt32 => {
                            parse_primitive_array::<UInt32Array, UInt32Type>(s)?
                        }
                        TypeInfoType::UInt64 => {
                            parse_primitive_array::<UInt64Array, UInt64Type>(s)?
                        }
                        // TypeInfoType::Float16 => {
                        //     parse_array_with_entities::<Float16Array, Float16Type>(s, e)?
                        // }
                        TypeInfoType::Float32 => {
                            parse_array_with_entities::<Float32Array, Float32Type>(s, e)?
                        }
                        TypeInfoType::Float64 => {
                            parse_array_with_entities::<Float64Array, Float64Type>(s, e)?
                        }
                        TypeInfoType::String => parse_string_array(s)?,
                        TypeInfoType::Bool => parse_bool_array(s)?,
                        TypeInfoType::Time => parse_timestamp_array(s)?,
                    };
                    Ok(arr)
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

/// Parse a JSON array containing elements of `U` into a mutable Arrow array `T`.
///
/// # Errors
///
/// Returns an error if the string is invalid JSON, if the elements of
/// the array are not of type `U`,  or if the Arrow buffer could not be
/// created.
fn parse_primitive_array<T, U>(s: &str) -> Result<Arc<dyn Array>, serde_json::Error>
where
    T: From<Vec<Option<U::Native>>> + Array + 'static,
    U: ArrowPrimitiveType,
    U::Native: DeserializeOwned,
{
    let v: Vec<Option<U::Native>> = serde_json::from_str(s)?;
    Ok(Arc::new(T::from(v)))
}

fn parse_timestamp_array(s: &str) -> Result<Arc<dyn Array>, serde_json::Error> {
    let v: Vec<Option<i64>> = serde_json::from_str::<Vec<Option<i64>>>(s)?
        .into_iter()
        .map(|opt| opt.map(|x| x * 1_000_000))
        .collect();
    Ok(Arc::new(TimestampNanosecondArray::from(v)))
}

fn parse_string_array(s: &str) -> Result<Arc<dyn Array>, serde_json::Error> {
    let v: Vec<Option<String>> = serde_json::from_str(s)?;
    Ok(Arc::new(StringArray::from(v)))
}

fn parse_bool_array(s: &str) -> Result<Arc<dyn Array>, serde_json::Error> {
    let v: Vec<Option<bool>> = serde_json::from_str(s)?;
    Ok(Arc::new(BooleanArray::from(v)))
}

/// Parse a JSON array containing elements of `U` into a mutable Arrow array `T`,
/// then substitutes any sentinel `entities`.
///
/// # Errors
///
/// Returns an error if the string is invalid JSON, if the elements of
/// the array are not of type `U`,  or if the Arrow buffer could not be
/// created.
///
/// # Panics
///
/// Panics if any of the indexes in `entities` are greater than the length
/// of the parsed array.
fn parse_array_with_entities<T, U>(
    s: &str,
    entities: Option<Entities>,
) -> Result<Arc<dyn Array>, serde_json::Error>
where
    T: From<Vec<Option<U::Native>>> + Array + 'static,
    U: ArrowPrimitiveType,
    U::Native: DeserializeOwned + Float,
{
    let de = DeArray::<T, U>::new(entities);
    let mut deserializer = serde_json::Deserializer::from_str(s);
    Ok(Arc::new(de.deserialize(&mut deserializer)?))
}

/// Helper struct used to deserialize a sequence into an Arrow `Array`.
#[derive(Debug)]
struct DeArray<T, U> {
    entities: Option<Entities>,
    // The type of the final array.
    t: PhantomData<T>,
    // The type of the elements in the array.
    u: PhantomData<U>,
}

impl<T, U> DeArray<T, U> {
    fn new(entities: Option<Entities>) -> Self {
        Self {
            entities,
            t: PhantomData,
            u: PhantomData,
        }
    }
}

// Deserialization for mutable Arrow arrays.
//
// Constructs a mutable array from a sequence of valid values.
// All of the `Mutable` variants of Arrow arrays implement `TryPush<Option<U>>`
// for some relevant `U`, and here we just impose that the `U` is `Deserialize`
// and gradually build up the array.
impl<'de, T, U> DeserializeSeed<'de> for DeArray<T, U>
where
    T: From<Vec<Option<U::Native>>> + Array + 'static,
    U: ArrowPrimitiveType,
    U::Native: Deserialize<'de> + Float,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor<T, U>(Option<Entities>, PhantomData<T>, PhantomData<U>);

        impl<'de, T, U> Visitor<'de> for ArrayVisitor<T, U>
        where
            T: From<Vec<Option<U::Native>>> + Array + 'static,
            U: ArrowPrimitiveType,
            U::Native: Deserialize<'de> + Float,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a heterogeneous array of compatible values")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut array = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(opt) = seq.next_element::<Option<U::Native>>()? {
                    array.push(opt);
                }

                if let Some(e) = self.0 {
                    e.nan.iter().for_each(|idx| {
                        if let Some(v) = array.get_mut(*idx) {
                            *v = Some(U::Native::nan());
                        }
                    });
                    e.inf.iter().for_each(|idx| {
                        if let Some(v) = array.get_mut(*idx) {
                            *v = Some(U::Native::infinity());
                        }
                    });
                    e.neg_inf.iter().for_each(|idx| {
                        if let Some(v) = array.get_mut(*idx) {
                            *v = Some(U::Native::neg_infinity());
                        }
                    });
                }
                Ok(array.into())
            }
        }

        deserializer.deserialize_seq(ArrayVisitor::<T, U>(
            self.entities,
            PhantomData,
            PhantomData,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::data::Frame;

    #[test]
    fn deserialize_golden() {
        let jdoc = include_str!("golden.json");
        let _: Frame = serde_json::from_str(jdoc).unwrap();
    }
}
