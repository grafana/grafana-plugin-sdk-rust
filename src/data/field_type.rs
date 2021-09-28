use arrow2::{
    array::{PrimitiveArray, Utf8Array},
    datatypes::{DataType, TimeUnit},
};
use chrono::{DateTime, Offset, TimeZone};

use crate::data::TypeInfoType;

pub trait FieldType {
    type Array;
    const ARROW_DATA_TYPE: DataType;
    const TYPE_INFO_TYPE: TypeInfoType;
}

pub trait IntoFieldType {
    type ElementType;
    fn into_field_type(self) -> Option<Self::ElementType>;
}

// Optional impl - a no-op.
impl<T> IntoFieldType for Option<T> {
    type ElementType = T;
    fn into_field_type(self) -> Option<Self::ElementType> {
        self
    }
}

macro_rules! impl_fieldtype_for_primitive {
    ($ty: ty, $arrow_ty: expr, $type_info: expr) => {
        impl FieldType for $ty {
            type Array = PrimitiveArray<$ty>;
            const ARROW_DATA_TYPE: DataType = $arrow_ty;
            const TYPE_INFO_TYPE: TypeInfoType = $type_info;
        }

        impl IntoFieldType for $ty {
            type ElementType = $ty;
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

// DateTime impls.

impl<T> FieldType for DateTime<T>
where
    T: Offset + TimeZone,
{
    type Array = PrimitiveArray<i64>;
    const ARROW_DATA_TYPE: DataType = DataType::Timestamp(TimeUnit::Nanosecond, None);
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::Time;
}

impl<T> IntoFieldType for DateTime<T>
where
    T: Offset + TimeZone,
{
    type ElementType = i64;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self.timestamp_nanos())
    }
}

// String impls.
impl FieldType for &str {
    type Array = Utf8Array<i32>;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
}

impl<'a> IntoFieldType for &'a str {
    type ElementType = &'a str;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}

impl FieldType for String {
    type Array = Utf8Array<i32>;
    const ARROW_DATA_TYPE: DataType = DataType::Utf8;
    const TYPE_INFO_TYPE: TypeInfoType = TypeInfoType::String;
}

impl IntoFieldType for String {
    type ElementType = String;
    fn into_field_type(self) -> Option<Self::ElementType> {
        Some(self)
    }
}
