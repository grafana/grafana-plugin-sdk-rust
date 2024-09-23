//! Types of field understood by the Grafana plugin SDK.
use std::time::{SystemTime, UNIX_EPOCH};

use arrow::{
    array::{BooleanArray, PrimitiveArray, StringArray},
    datatypes::{
        DataType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type, TimeUnit,
        TimestampNanosecondType, UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
};
use chrono::prelude::*;

use crate::data::TypeInfoType;

/// Indicates that a type is can be stored in an Arrow array.
pub trait FieldType {
    /// The type of arrow array this field type is stored in.
    type InArray;
    /// The type of arrow array to convert to when storing values in a `Field`.
    type OutArray;
    /// The logical arrow data type that an arrow array of this data should have.
    const ARROW_DATA_TYPE: DataType;

    /// Convert the logical type of `Self::InArray`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray;
}

/// Indicates that a type can be converted to one that is [`FieldType`], and holds associated metadata.
///
/// For example, [`DateTime<T>`]s are valid `Field` values, but must first be converted
/// to nanoseconds-since-the-epoch in i64 values; the original type and corresponding
/// [`TypeInfoType`] is stored here.
///
/// This trait mainly exists to enable smoother APIs when creating [`Field`]s.
///
/// [`Field`]: crate::data::Field
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
    ($ty: ty, $arrow_ty: ty, $arrow_data_ty: expr, $type_info: expr) => {
        impl FieldType for $ty {
            type InArray = PrimitiveArray<$arrow_ty>;
            type OutArray = PrimitiveArray<$arrow_ty>;
            const ARROW_DATA_TYPE: DataType = $arrow_data_ty;
            fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
                array
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

impl_fieldtype_for_primitive!(i8, Int8Type, DataType::Int8, TypeInfoType::Int8);
impl_fieldtype_for_primitive!(i16, Int16Type, DataType::Int16, TypeInfoType::Int16);
impl_fieldtype_for_primitive!(i32, Int32Type, DataType::Int32, TypeInfoType::Int32);
impl_fieldtype_for_primitive!(i64, Int64Type, DataType::Int64, TypeInfoType::Int64);
impl_fieldtype_for_primitive!(u8, UInt8Type, DataType::UInt8, TypeInfoType::UInt8);
impl_fieldtype_for_primitive!(u16, UInt16Type, DataType::UInt16, TypeInfoType::UInt16);
impl_fieldtype_for_primitive!(u32, UInt32Type, DataType::UInt32, TypeInfoType::UInt32);
impl_fieldtype_for_primitive!(u64, UInt64Type, DataType::UInt64, TypeInfoType::UInt64);
impl_fieldtype_for_primitive!(f32, Float32Type, DataType::Float32, TypeInfoType::Float32);
impl_fieldtype_for_primitive!(f64, Float64Type, DataType::Float64, TypeInfoType::Float64);

// Boolean impl.

impl FieldType for bool {
    type InArray = BooleanArray;
    type OutArray = BooleanArray;
    const ARROW_DATA_TYPE: DataType = DataType::Boolean;

    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
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
    type InArray = PrimitiveArray<Int64Type>;
    type OutArray = PrimitiveArray<TimestampNanosecondType>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array.reinterpret_cast()
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
    type InArray = PrimitiveArray<Int64Type>;
    type OutArray = PrimitiveArray<TimestampNanosecondType>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array.reinterpret_cast()
    }
}

impl<T> IntoFieldType for DateTime<T>
where
    T: Offset + TimeZone,
{
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self.timestamp_nanos_opt()
    }
}

// The `Date` type was only recently deprecated, we should remove support for it in the next
// minor version.
#[allow(deprecated)]
impl<T> FieldType for Date<T>
where
    T: Offset + TimeZone,
{
    type InArray = PrimitiveArray<Int64Type>;
    type OutArray = PrimitiveArray<TimestampNanosecondType>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array.reinterpret_cast()
    }
}

// The `Date` type was only recently deprecated, we should remove support for it in the next
// minor version.
#[allow(deprecated)]
impl<T> IntoFieldType for Date<T>
where
    T: Offset + TimeZone,
{
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self.and_hms_opt(0, 0, 0)
            .expect("hms are valid")
            .timestamp_nanos_opt()
    }
}

impl FieldType for NaiveDate {
    type InArray = PrimitiveArray<Int64Type>;
    type OutArray = PrimitiveArray<TimestampNanosecondType>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array.reinterpret_cast()
    }
}

impl IntoFieldType for NaiveDate {
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self.and_hms_opt(0, 0, 0)
            .expect("hms are valid")
            .and_utc()
            .timestamp_nanos_opt()
    }
}

impl FieldType for NaiveDateTime {
    type InArray = PrimitiveArray<Int64Type>;
    type OutArray = PrimitiveArray<TimestampNanosecondType>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);

    /// Convert the logical type of `Self::Array` to `DataType::Timestamp`.
    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array.reinterpret_cast()
    }
}

impl IntoFieldType for NaiveDateTime {
    type ElementType = i64;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self.and_utc().timestamp_nanos_opt()
    }
}

// String impls.

impl FieldType for &str {
    type InArray = StringArray;
    type OutArray = StringArray;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;

    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array
    }
}

impl<'a> IntoFieldType for &'a str {
    type ElementType = &'a str;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}

impl FieldType for String {
    type InArray = StringArray;
    type OutArray = StringArray;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;

    fn convert_arrow_array(array: Self::InArray) -> Self::OutArray {
        array
    }
}

impl IntoFieldType for String {
    type ElementType = String;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}
