//! Deserialization of [`Frame`]s from the JSON format.
use std::{collections::BTreeMap, fmt, marker::PhantomData};

use arrow2::{
    array::{
        Array, MutableArray, MutableBooleanArray, MutablePrimitiveArray, MutableUtf8Array, TryPush,
    },
    datatypes::{DataType, TimeUnit},
    types::{NativeType, Offset},
};
use num_traits::Float;
use serde::{
    de::{Deserializer, Error, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use serde_json::from_str;

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
    values: Vec<Box<dyn Array>>,
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
                    let arr: Box<dyn Array> = match f.type_info.frame {
                        TypeInfoType::Int8 => {
                            parse_array::<MutablePrimitiveArray<i8>, i8, ()>(s)?.as_box()
                        }
                        TypeInfoType::Int16 => {
                            parse_array::<MutablePrimitiveArray<i16>, i16, ()>(s)?.as_box()
                        }
                        TypeInfoType::Int32 => {
                            parse_array::<MutablePrimitiveArray<i32>, i32, ()>(s)?.as_box()
                        }
                        TypeInfoType::Int64 => {
                            parse_array::<MutablePrimitiveArray<i64>, i64, ()>(s)?.as_box()
                        }
                        TypeInfoType::UInt8 => {
                            parse_array::<MutablePrimitiveArray<u8>, u8, ()>(s)?.as_box()
                        }
                        TypeInfoType::UInt16 => {
                            parse_array::<MutablePrimitiveArray<u16>, u16, ()>(s)?.as_box()
                        }
                        TypeInfoType::UInt32 => {
                            parse_array::<MutablePrimitiveArray<u32>, u32, ()>(s)?.as_box()
                        }
                        TypeInfoType::UInt64 => {
                            parse_array::<MutablePrimitiveArray<u64>, u64, ()>(s)?.as_box()
                        }
                        TypeInfoType::Float32 => {
                            parse_array_with_entities::<MutablePrimitiveArray<f32>, f32>(s, e)?
                                .as_box()
                        }
                        TypeInfoType::Float64 => {
                            parse_array_with_entities::<MutablePrimitiveArray<f64>, f64>(s, e)?
                                .as_box()
                        }
                        TypeInfoType::String => {
                            parse_array::<MutableUtf8Array<i32>, String, ()>(s)?.as_box()
                        }
                        TypeInfoType::Bool => {
                            parse_array::<MutableBooleanArray, bool, ()>(s)?.as_box()
                        }
                        TypeInfoType::Time => {
                            parse_array::<MutablePrimitiveArray<i64>, i64, TimestampProcessor>(s)?
                                .to(DataType::Timestamp(TimeUnit::Nanosecond, None))
                                .as_box()
                        }
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
fn parse_array<'de, T, U, V>(s: &'de str) -> Result<T, serde_json::Error>
where
    T: Default + MutableArray + TryPush<Option<U>> + WithCapacity,
    U: Deserialize<'de>,
    V: ElementProcessor<U>,
{
    Ok(from_str::<DeArray<T, U, V>>(s)?.array)
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
fn parse_array_with_entities<'de, T, U>(
    s: &'de str,
    entities: Option<Entities>,
) -> Result<T, serde_json::Error>
where
    T: Default + MutableArray + SetArray<Option<U>> + TryPush<Option<U>> + WithCapacity,
    U: Deserialize<'de> + Float,
{
    let mut arr = from_str::<DeArray<T, U>>(s)?.array;
    if let Some(e) = entities {
        e.nan.iter().for_each(|idx| arr.set(*idx, Some(U::nan())));
        e.inf
            .iter()
            .for_each(|idx| arr.set(*idx, Some(U::infinity())));
        e.neg_inf
            .iter()
            .for_each(|idx| arr.set(*idx, Some(U::neg_infinity())));
    }
    Ok(arr)
}

trait ElementProcessor<T> {
    fn process_element(el: T) -> T {
        el
    }
}

impl<T> ElementProcessor<T> for () {}

struct TimestampProcessor;
impl ElementProcessor<i64> for TimestampProcessor {
    fn process_element(el: i64) -> i64 {
        el * 1_000_000
    }
}

/// Helper struct used to deserialize a sequence into an Arrow `Array`.
#[derive(Debug)]
struct DeArray<T, U, V = ()> {
    array: T,
    u: PhantomData<U>,
    v: PhantomData<V>,
}

// Deserialization for mutable Arrow arrays.
//
// Constructs a mutable array from a sequence of valid values.
// All of the `Mutable` variants of Arrow arrays implement `TryPush<Option<U>>`
// for some relevant `U`, and here we just impose that the `U` is `Deserialize`
// and gradually build up the array.
impl<'de, T, U, V> Deserialize<'de> for DeArray<T, U, V>
where
    T: Default + TryPush<Option<U>> + WithCapacity,
    U: Deserialize<'de>,
    V: ElementProcessor<U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor<T, U, V>(PhantomData<T>, PhantomData<U>, PhantomData<V>);

        impl<'de, T, U, V> Visitor<'de> for ArrayVisitor<T, U, V>
        where
            T: Default + TryPush<Option<U>> + WithCapacity,
            U: Deserialize<'de>,
            V: ElementProcessor<U>,
        {
            type Value = DeArray<T, U, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a heterogeneous array of compatible values")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut array = seq.size_hint().map_or_else(T::default, T::with_capacity);
                while let Some(x) = seq.next_element::<Option<U>>()? {
                    array.try_push(x.map(V::process_element)).map_err(|e| {
                        S::Error::custom(format!("could not push to Arrow array: {}", e))
                    })?;
                }
                Ok(Self::Value {
                    array,
                    u: PhantomData,
                    v: PhantomData,
                })
            }
        }

        deserializer.deserialize_seq(ArrayVisitor(PhantomData, PhantomData, PhantomData))
    }
}

/// Indicates that an array can have its elements mutated.
trait SetArray<T> {
    /// Set the value at `index` to `value`.
    ///
    /// Note that if it is the first time a null appears in this array,
    /// this initializes the validity bitmap (`O(N)`).
    ///
    /// # Panics
    ///
    /// Panics iff index is larger than `self.len()`.
    fn set(&mut self, index: usize, value: T);
}

impl<T: NativeType> SetArray<Option<T>> for MutablePrimitiveArray<T> {
    fn set(&mut self, index: usize, value: Option<T>) {
        self.set(index, value);
    }
}

/// Indicates that an array can be created with a specified capacity.
trait WithCapacity {
    /// Create `Self` with the given `capacity` preallocated.
    ///
    /// This generally just delegates to the `with_capacity` method on
    /// individual arrays.
    fn with_capacity(capacity: usize) -> Self;
}

impl<T: NativeType> WithCapacity for MutablePrimitiveArray<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
}

impl WithCapacity for MutableBooleanArray {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
}

impl<T: Offset> WithCapacity for MutableUtf8Array<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
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
