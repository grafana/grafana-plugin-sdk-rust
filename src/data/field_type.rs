//! Types of field understood by the Grafana plugin SDK.
use std::time::{SystemTime, UNIX_EPOCH};

use arrow2::{
    array::{BooleanArray, PrimitiveArray, Utf8Array},
    datatypes::{DataType, TimeUnit},
};
use chrono::prelude::*;

use crate::data::TypeInfoType;

/// Indicates that a type is can be stored in an Arrow array.
pub trait FieldType {
    /// The type of arrow array this field type is stored in.
    type Array;
    /// The logical arrow data type that an arrow array of this data should have.
    const ARROW_DATA_TYPE: DataType;

    /// Convert the logical type of `Self::Array`, if needed.
    ///
    /// The default implementation is a no-op, but some field types may need to
    /// implement this to ensure the underlying boxed Arrow array can be downcast correctly.
    fn convert_arrow_array(array: Self::Array, _data_type: DataType) -> Self::Array {
        array
    }
}

/// Indicates that a type can be converted to one that is [`FieldType`], and holds associated metadata.
///
/// For example, [`DateTime<T>`]s are valid `Field` values, but must first be converted
/// to nanoseconds-since-the-epoch in i64 values; the original type and corresponding
/// [`TypeInfoType`] is stored here.
///
/// This trait mainly exists to enable smoother APIs when creating [`Field`]s.
pub trait IntoFieldType {
    /// The type to which `Self` will be converted when storing values in a `Field`.
    type ElementType;
    /// The corresponding [`TypeInfoType`] for this original data type.
    const TYPE_INFO_TYPE: TypeInfoType;
    /// Convert this type into an (optional) field type.
    fn into_field_type(self) -> Option<Self::ElementType>;
}

// Optional impl - a no-op.
impl<T> IntoFieldType for Option<T>
where
    T: IntoFieldType,
{
    type ElementType = T;
    const TYPE_INFO_TYPE: TypeInfoType = T::TYPE_INFO_TYPE;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self
    }
}

macro_rules! impl_fieldtype_for_primitive {
    ($ty: ty, $arrow_ty: expr, $type_info: expr) => {
        impl FieldType for $ty {
            type Array = PrimitiveArray<$ty>;
            const ARROW_DATA_TYPE: DataType = $arrow_ty;
            fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
                array.to(data_type)
            }
        }

        impl IntoFieldType for $ty {
            type ElementType = $ty;
            const TYPE_INFO_TYPE: TypeInfoType = $type_info;
            fn into_field_type(self) -> Option<Self::ElementType> {
                Some(self)
            }
        }
    };
}

impl_fieldtype_for_primitive!(i8, DataType::Int8, TypeInfoType::Int8);
impl_fieldtype_for_primitive!(i16, DataType::Int16, TypeInfoType::Int16);
impl_fieldtype_for_primitive!(i32, DataType::Int32, TypeInfoType::Int32);
impl_fieldtype_for_primitive!(i64, DataType::Int64, TypeInfoType::Int64);
impl_fieldtype_for_primitive!(u8, DataType::UInt8, TypeInfoType::UInt8);
impl_fieldtype_for_primitive!(u16, DataType::UInt16, TypeInfoType::UInt16);
impl_fieldtype_for_primitive!(u32, DataType::UInt32, TypeInfoType::UInt32);
impl_fieldtype_for_primitive!(u64, DataType::UInt64, TypeInfoType::UInt64);
impl_fieldtype_for_primitive!(f32, DataType::Float32, TypeInfoType::Float32);
impl_fieldtype_for_primitive!(f64, DataType::Float64, TypeInfoType::Float64);

// Boolean impl.

impl FieldType for bool {
    type Array = BooleanArray;
    const ARROW_DATA_TYPE: DataType = DataType::Boolean;

    fn convert_arrow_array(array: Self::Array, _data_type: DataType) -> Self::Array {
        array
    }
}

impl IntoFieldType for bool {
    type ElementType = bool;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Bool;

    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}

// DateTime impls.

impl FieldType for SystemTime {
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
        array.to(data_type)
    }
}

impl IntoFieldType for SystemTime {
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        // This won't overflow for about 300 years so we're probably fine.
        self.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|x| x.as_nanos().try_into().ok())
    }
}

impl<T> FieldType for DateTime<T>
where
    T: Offset + TimeZone,
{
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
        array.to(data_type)
    }
}

impl<T> IntoFieldType for DateTime<T>
where
    T: Offset + TimeZone,
{
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self.timestamp_nanos())
    }
}

impl<T> FieldType for Date<T>
where
    T: Offset + TimeZone,
{
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
        array.to(data_type)
    }
}

impl<T> IntoFieldType for Date<T>
where
    T: Offset + TimeZone,
{
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self.and_hms(0, 0, 0).timestamp_nanos())
    }
}

impl FieldType for NaiveDate {
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
        array.to(data_type)
    }
}

impl IntoFieldType for NaiveDate {
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self.and_hms(0, 0, 0).timestamp_nanos())
    }
}

impl FieldType for NaiveDateTime {
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::Array, data_type: DataType) -> Self::Array {
        array.to(data_type)
    }
}

impl IntoFieldType for NaiveDateTime {
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self.timestamp_nanos())
    }
}

// String impls.

impl FieldType for &str {
    type Array = Utf8Array<i32>;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;
}

impl<'a> IntoFieldType for &'a str {
    type ElementType = &'a str;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}

impl FieldType for String {
    type Array = Utf8Array<i32>;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;
}

impl IntoFieldType for String {
    type ElementType = String;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}
