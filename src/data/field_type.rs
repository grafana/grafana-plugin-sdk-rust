use arrow2::{
    array::{PrimitiveArray, Utf8Array},
    datatypes::DataType,
};
use chrono::{DateTime, Offset, TimeZone};

use crate::data::TypeInfoType;

pub trait FieldType {
    type Array;
    const ARROW_DATA_TYPE: DataType;

    /// Convert the logical type of `Self::Array`, if needed.
    ///
    /// The default implementation is a no-op, but some field types
    /// may need to implement this.
    fn convert_arrow_array(array: Self::Array, _data_type: DataType) -> Self::Array {
        array
    }
}

pub trait IntoFieldType {
    type ElementType;
    const TYPE_INFO_TYPE: TypeInfoType;
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

// DateTime impls.

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
