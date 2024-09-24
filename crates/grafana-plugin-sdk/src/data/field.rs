//! Contains the `Field` struct, which holds actual data in the form of Arrow arrays, as well as column-specific metadata.
use std::{
    collections::{BTreeMap, HashMap},
    iter::FromIterator,
    sync::Arc,
};

use arrow::{
    array::Array,
    datatypes::{DataType, Field as ArrowField, TimeUnit},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::data::{
    error,
    field_type::{FieldType, IntoFieldType},
    ConfFloat64,
};

/// A typed column within a [`Frame`][crate::data::Frame].
///
/// The underlying data for this field can be read using the [`Field::values`] method,
/// and updated using the [`Field::set_values`] and [`Field::set_values_opt`] methods.
#[derive(Debug)]
pub struct Field {
    /// The name of this field.
    ///
    /// Fields within a [`Frame`][crate::data::Frame] are not required to have unique names, but
    /// the combination of `name` and `labels` should be unique within a frame
    /// to ensure proper behaviour in all situations.
    pub name: String,
    /// An optional set of key-value pairs that, combined with the name, should uniquely identify a field within a [`Frame`][crate::data::Frame].
    pub labels: BTreeMap<String, String>,
    /// Optional display configuration used by Grafana.
    pub config: Option<FieldConfig>,

    /// The actual values of this field.
    ///
    /// The types of values contained within the `Array` MUST match the
    /// type information in `type_info` at all times. The various `into_field`-like
    /// functions and the `set_values` methods should ensure this.
    pub(crate) values: Arc<dyn Array>,
    /// Type information for this field, as understood by Grafana.
    pub(crate) type_info: TypeInfo,
}

impl Field {
    pub(crate) fn to_arrow_field(&self) -> Result<ArrowField, serde_json::Error> {
        let metadata = match (self.labels.is_empty(), self.config.as_ref()) {
            (true, None) => Default::default(),
            (false, None) => [("labels".to_string(), serde_json::to_string(&self.labels)?)]
                .into_iter()
                .collect(),
            (false, Some(c)) => [("config".to_string(), serde_json::to_string(&c)?)]
                .into_iter()
                .collect(),
            (true, Some(c)) => [
                ("labels".to_string(), serde_json::to_string(&self.labels)?),
                ("config".to_string(), serde_json::to_string(&c)?),
            ]
            .into_iter()
            .collect(),
        };
        Ok(ArrowField::new(
            self.name.clone(),
            self.type_info.frame.into(),
            self.type_info.nullable.unwrap_or_default(),
        )
        .with_metadata(metadata))
    }

    /// Return a new field with the given name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let field = ["a", "b", "c"]
    ///     .into_field("x")
    ///     .with_name("other name");
    /// assert_eq!(&field.name, "other name");
    /// ```
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Return a new field with the given labels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::BTreeMap;
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut labels = BTreeMap::default();
    /// labels.insert("some".to_string(), "value".to_string());
    /// let field = ["a", "b", "c"]
    ///     .into_field("x")
    ///     .with_labels(labels);
    /// assert_eq!(field.labels["some"], "value");
    /// ```
    #[must_use]
    pub fn with_labels(mut self, labels: BTreeMap<String, String>) -> Self {
        self.labels = labels;
        self
    }

    /// Return a new field with the given config.
    ///
    /// # Example
    ///
    /// ```rust
    /// use grafana_plugin_sdk::{data::FieldConfig, prelude::*};
    ///
    /// let mut config = FieldConfig::default();
    /// config.display_name_from_ds = Some("X".to_string());
    /// let field = ["a", "b", "c"]
    ///     .into_field("x")
    ///     .with_config(config);
    /// assert_eq!(&field.config.unwrap().display_name_from_ds.unwrap(), "X");
    /// ```
    #[must_use]
    pub fn with_config(mut self, config: impl Into<Option<FieldConfig>>) -> Self {
        self.config = config.into();
        self
    }

    /// Get the values of this field as a [`&dyn Array`].
    pub fn values(&self) -> &dyn Array {
        &*self.values
    }

    /// Set the values of this field using an iterator of values.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::DataTypeMismatch`][error::Error::DataTypeMismatch] if the types of the new data
    /// do not match the types of the existing data.
    ///
    /// ```rust
    /// use arrow::array::StringArray;
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut field = ["a", "b", "c"]
    ///     .into_field("x");
    /// assert!(field.set_values(["d", "e", "f", "g"]).is_ok());
    /// assert_eq!(
    ///     field
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<StringArray>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<_>>(),
    ///     vec![Some("d"), Some("e"), Some("f"), Some("g")],
    /// );
    ///
    /// assert!(field.set_values([1u32, 2, 3]).is_err());
    /// ```
    pub fn set_values<T, U, V>(&mut self, values: T) -> Result<(), crate::data::error::Error>
    where
        T: IntoIterator<Item = U>,
        U: IntoFieldType<ElementType = V>,
        V: FieldType,
        V::InArray: Array + FromIterator<Option<V>> + 'static,
        V::OutArray: Array + FromIterator<Option<V>> + 'static,
    {
        let new_data_type: DataType = U::TYPE_INFO_TYPE.into();
        if self.values.data_type() != &new_data_type {
            return Err(crate::data::error::Error::DataTypeMismatch {
                existing: self.values.data_type().clone(),
                new: new_data_type,
                field: self.name.clone(),
            });
        }
        self.values = Arc::new(V::convert_arrow_array(
            values
                .into_iter()
                .map(U::into_field_type)
                .collect::<V::InArray>(),
        ));
        self.type_info.nullable = Some(false);
        Ok(())
    }

    /// Set the values of this field using an iterator of optional values.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::DataTypeMismatch`][error::Error::DataTypeMismatch] if the types of the new data
    /// do not match the types of the existing data.
    ///
    /// ```rust
    /// use arrow::array::StringArray;
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut field = ["a", "b", "c"]
    ///     .into_field("x");
    /// assert!(field.set_values_opt([Some("d"), Some("e"), None, None]).is_ok());
    /// assert_eq!(
    ///     field
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<StringArray>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<_>>(),
    ///     vec![Some("d"), Some("e"), None, None],
    /// );
    ///
    /// assert!(field.set_values([Some(1u32), Some(2), None]).is_err());
    /// ```
    pub fn set_values_opt<T, U, V>(&mut self, values: T) -> Result<(), crate::data::error::Error>
    where
        T: IntoIterator<Item = Option<U>>,
        U: IntoFieldType<ElementType = V>,
        V: FieldType,
        V::InArray: Array + FromIterator<Option<V>> + 'static,
        V::OutArray: Array + FromIterator<Option<V>> + 'static,
    {
        let new_data_type: DataType = U::TYPE_INFO_TYPE.into();
        if self.values.data_type() != &new_data_type {
            return Err(crate::data::error::Error::DataTypeMismatch {
                existing: self.values.data_type().clone(),
                new: new_data_type,
                field: self.name.clone(),
            });
        }
        self.values = Arc::new(V::convert_arrow_array(
            values
                .into_iter()
                .map(|x| x.and_then(U::into_field_type))
                .collect::<V::InArray>(),
        ));
        self.type_info.nullable = Some(true);
        Ok(())
    }

    /// Set the values of this field using an [`Array`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error::DataTypeMismatch`][error::Error::DataTypeMismatch] if the types of the new data
    /// do not match the types of the existing data.
    ///
    /// ```rust
    /// use arrow::array::{PrimitiveArray, StringArray};
    /// use grafana_plugin_sdk::prelude::*;
    ///
    /// let mut field = ["a", "b", "c"]
    ///     .into_field("x");
    /// let new_values = StringArray::from(["d", "e", "f"].map(Some).to_vec());
    /// assert!(field.set_values_array(new_values).is_ok());
    /// assert_eq!(
    ///     field
    ///         .values()
    ///         .as_any()
    ///         .downcast_ref::<StringArray>()
    ///         .unwrap()
    ///         .iter()
    ///         .collect::<Vec<_>>(),
    ///     vec![Some("d"), Some("e"), Some("f")],
    /// );
    ///
    /// let bad_values = PrimitiveArray::from([1u32, 2, 3].map(Some).to_vec());
    /// assert!(field.set_values_array(bad_values).is_err());
    /// ```
    pub fn set_values_array<T>(&mut self, values: T) -> Result<(), crate::data::error::Error>
    where
        T: Array + 'static,
    {
        if self.values.data_type() != values.data_type() {
            return Err(crate::data::error::Error::DataTypeMismatch {
                existing: self.values.data_type().clone(),
                new: values.data_type().clone(),
                field: self.name.clone(),
            });
        }
        self.values = Arc::new(values);
        Ok(())
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.labels == other.labels
            && self.config == other.config
            && self.type_info == other.type_info
            && other.values.eq(&self.values)
    }
}

// Traits for creating a `Field` from various array-like things:
// iterators of both values and optional values, and arrays themselves.
// These need to be separate traits because otherwise the impls would conflict,
// as e.g. `Array` implements `IntoIterator`.

/// Indicates that a [`Field`] can be created from this type.
pub trait IntoField {
    /// Create a [`Field`] from `self`.
    ///
    /// The type of the `Field` will depend on the values in `self`.
    fn into_field(self, name: impl Into<String>) -> Field;
}

impl<T, U, V> IntoField for T
where
    T: IntoIterator<Item = U>,
    U: FieldType + IntoFieldType<ElementType = V>,
    U::InArray: Array + FromIterator<Option<V>> + 'static,
    U::OutArray: Array + FromIterator<Option<V>> + 'static,
{
    fn into_field(self, name: impl Into<String>) -> Field {
        Field {
            name: name.into(),
            labels: Default::default(),
            config: None,
            type_info: TypeInfo {
                frame: U::TYPE_INFO_TYPE,
                nullable: Some(false),
            },
            values: Arc::new(U::convert_arrow_array(
                self.into_iter()
                    .map(U::into_field_type)
                    .collect::<U::InArray>(),
            )),
        }
    }
}

/// Indicates that a [`Field`] of optional values can be created from this type.
pub trait IntoOptField {
    /// Create a [`Field`] from `self`, with `None` values marked as null.
    fn into_opt_field(self, name: impl Into<String>) -> Field;
}

impl<T, U, V> IntoOptField for T
where
    T: IntoIterator<Item = Option<U>>,
    U: FieldType + IntoFieldType<ElementType = V>,
    U::InArray: Array + FromIterator<Option<V>> + 'static,
    U::OutArray: Array + FromIterator<Option<V>> + 'static,
{
    fn into_opt_field(self, name: impl Into<String>) -> Field {
        Field {
            name: name.into(),
            labels: Default::default(),
            config: None,
            type_info: TypeInfo {
                frame: U::TYPE_INFO_TYPE,
                nullable: Some(true),
            },
            values: Arc::new(U::convert_arrow_array(
                self.into_iter()
                    .map(|x| x.and_then(U::into_field_type))
                    .collect::<U::InArray>(),
            )),
        }
    }
}

/// Helper trait for creating a [`Field`] from an [`Array`].
pub trait ArrayIntoField {
    /// Create a `Field` using `self` as the values.
    ///
    /// # Errors
    ///
    /// This returns an error if the values are not valid field types.
    fn try_into_field(self, name: impl Into<String>) -> Result<Field, error::Error>;
}

impl<T> ArrayIntoField for T
where
    T: Array + 'static,
{
    fn try_into_field(self, name: impl Into<String>) -> Result<Field, error::Error> {
        Ok(Field {
            name: name.into(),
            labels: Default::default(),
            config: None,
            type_info: TypeInfo {
                frame: self.data_type().try_into()?,
                nullable: Some(true),
            },
            values: Arc::new(self),
        })
    }
}

/// Helper trait for creating a [`Field`] from an [`Array`].
pub trait ArrayRefIntoField {
    /// Create a `Field` using `self` as the values.
    ///
    /// # Errors
    ///
    /// This returns an error if the values are not valid field types.
    fn try_into_field(self, name: impl Into<String>) -> Result<Field, error::Error>;
}

impl ArrayRefIntoField for Arc<dyn arrow::array::Array> {
    fn try_into_field(self, name: impl Into<String>) -> Result<Field, error::Error> {
        Ok(Field {
            name: name.into(),
            labels: Default::default(),
            config: None,
            type_info: TypeInfo {
                frame: self.data_type().try_into()?,
                nullable: Some(true),
            },
            values: self,
        })
    }
}

/// The type information for a [`Frame`][crate::data::Frame] as understood by Grafana.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeInfo {
    /// The type, as understood by Grafana.
    pub(crate) frame: TypeInfoType,
    /// Is this type nullable?
    #[serde(default)]
    pub(crate) nullable: Option<bool>,
}

impl TypeInfo {
    /// Get the corresponding [`SimpleType`].
    #[must_use]
    pub const fn simple_type(&self) -> SimpleType {
        self.frame.simple_type()
    }
}

/// Valid types understood by Grafana.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeInfoType {
    /// An 8 bit signed integer.
    Int8,
    /// A 16 bit signed integer.
    Int16,
    /// A 32 bit signed integer.
    Int32,
    /// A 64 bit signed integer.
    Int64,
    /// An 8 bit unsigned integer.
    UInt8,
    /// A 16 bit unsigned integer.
    UInt16,
    /// A 32 bit unsigned integer.
    UInt32,
    /// A 64 bit unsigned integer.
    UInt64,
    /// A 32 bit float.
    Float32,
    /// A 64 bit float.
    Float64,
    /// A string.
    String,
    /// A boolean.
    Bool,
    /// A timestamp, in UTC.
    // For compatibility with the Go SDK.
    #[serde(rename = "time.Time")]
    Time,
}

impl TryFrom<&DataType> for TypeInfoType {
    type Error = error::Error;
    fn try_from(other: &DataType) -> Result<Self, Self::Error> {
        Ok(match other {
            DataType::Int8 => Self::Int8,
            DataType::Int16 => Self::Int16,
            DataType::Int32 => Self::Int32,
            DataType::Int64 => Self::Int64,
            DataType::UInt8 => Self::UInt8,
            DataType::UInt16 => Self::UInt16,
            DataType::UInt32 => Self::UInt32,
            DataType::UInt64 => Self::UInt64,
            DataType::Float32 => Self::Float32,
            DataType::Float64 => Self::Float64,
            DataType::Utf8 => Self::String,
            DataType::Boolean => Self::Bool,
            DataType::Timestamp(..) => Self::Time,
            // TODO - handle time correctly.
            other => return Err(error::Error::UnsupportedArrowDataType(other.clone())),
        })
    }
}

impl From<TypeInfoType> for DataType {
    fn from(other: TypeInfoType) -> Self {
        match other {
            TypeInfoType::Int8 => Self::Int8,
            TypeInfoType::Int16 => Self::Int16,
            TypeInfoType::Int32 => Self::Int32,
            TypeInfoType::Int64 => Self::Int64,
            TypeInfoType::UInt8 => Self::UInt8,
            TypeInfoType::UInt16 => Self::UInt16,
            TypeInfoType::UInt32 => Self::UInt32,
            TypeInfoType::UInt64 => Self::UInt64,
            TypeInfoType::Float32 => Self::Float32,
            TypeInfoType::Float64 => Self::Float64,
            TypeInfoType::String => Self::Utf8,
            TypeInfoType::Bool => Self::Boolean,
            TypeInfoType::Time => Self::Timestamp(TimeUnit::Nanosecond, None),
        }
    }
}

impl TypeInfoType {
    #[must_use]
    pub(crate) const fn simple_type(&self) -> SimpleType {
        match self {
            Self::Int8
            | Self::Int16
            | Self::Int32
            | Self::Int64
            | Self::UInt8
            | Self::UInt16
            | Self::UInt32
            | Self::UInt64
            | Self::Float32
            | Self::Float64 => SimpleType::Number,
            Self::String => SimpleType::String,
            Self::Bool => SimpleType::Boolean,
            Self::Time => SimpleType::Time,
        }
    }
}

/// The 'simple' type of this data.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SimpleType {
    /// A number.
    Number,
    /// A boolean.
    Boolean,
    /// A string.
    String,
    /// A timestamp.
    Time,
}

/// The display properties for a [`Field`].
///
/// These are used by the Grafana frontend to modify how the field is displayed.
///
/// Note that this struct, like most structs in this crate, is marked `#[non_exhaustive]` and
/// therefore cannot be constructed using a struct expression. Instead, create a default
/// value using `FieldConfig::default()` and modify any fields necessary.
// This struct needs to match the frontend component defined in:
// https://github.com/grafana/grafana/blob/master/packages/grafana-data/src/types/dataFrame.ts#L23
// All properties are optional should be omitted from JSON when empty or not set.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FieldConfig {
    /// Overrides Grafana default naming.
    ///
    /// This should not be used from a datasource.
    #[serde(default)]
    pub display_name: Option<String>,

    /// Overrides Grafana default naming in a better way that allows users to further override it easily.
    ///
    /// This should be used instead of `display_name` when used from a datasource.
    #[serde(default, rename = "displayNameFromDS")]
    pub display_name_from_ds: Option<String>,

    /// An explicit path to the field in the datasource.
    ///
    /// When the frame meta includes a path, this will default to `${frame.meta.path}/${field.name}.
    ///
    /// When defined, this value can be used as an identifier within the datasource scope, and
    /// may be used as an identifier to update values in a subsequent request.
    #[serde(default)]
    pub path: Option<String>,

    /// Human readable field metadata.
    #[serde(default)]
    pub description: Option<String>,

    /// Indicates that the datasource knows how to update this value.
    #[serde(default)]
    pub writeable: Option<bool>,

    /// Indicates if the field's data can be filtered by additional calls.
    #[serde(default)]
    pub filterable: Option<bool>,

    /// The string to display to represent this field's unit, such as "Requests/sec".
    #[serde(default)]
    pub unit: Option<String>,
    /// The number of decimal places to display.
    #[serde(default)]
    pub decimals: Option<u16>,
    /// The minimum value of fields in the column.
    ///
    /// When present the frontend can skip the calculation.
    #[serde(default)]
    pub min: Option<f64>,
    /// The maximum value of fields in the column.
    ///
    /// When present the frontend can skip the calculation.
    #[serde(default)]
    pub max: Option<f64>,

    /// Mappings from input value to display string.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<ValueMapping>,
    /// Mappings from numeric values to states.
    #[serde(default)]
    pub thresholds: Option<ThresholdsConfig>,

    /// Links to use when clicking on a result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DataLink>,

    /// An alternative string to use when no value is present.
    ///
    /// The default is an empty string.
    #[serde(default)]
    pub no_value: Option<String>,

    /// Panel-specific values.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, Value>,
}

/// Special values that can be mapped to new text and colour values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
#[non_exhaustive]
pub enum SpecialValueMatch {
    /// Match `true`.
    True,
    /// Match `false`.
    False,
    /// Match `null`.
    Null,
    /// Match `NaN`.
    NaN,
    /// Match `null` and `NaN`.
    #[serde(rename = "null+nan")]
    NullAndNaN,
    /// Match empty values.
    Empty,
}

/// Allows input values to be mapped to text and colour.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "options", rename_all = "camelCase")]
#[non_exhaustive]
pub enum ValueMapping {
    /// Map strings to new strings directly.
    ValueMapper(HashMap<String, ValueMappingResult>),
    /// Map special values to new values.
    SpecialValueMapper {
        /// The input value to match.
        #[serde(rename = "match")]
        match_: SpecialValueMatch,
        /// The result to be shown instead of the matched value.
        result: ValueMappingResult,
    },
    /// Map ranges of floats to new values.
    RangeValueMapper {
        /// The minimum value to match.
        from: ConfFloat64,
        /// The maximum value to match.
        to: ConfFloat64,
        /// The result to be shown instead of the matched values.
        result: ValueMappingResult,
    },
}

/// A new value to be displayed when a [`ValueMapping`] matches an input value.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ValueMappingResult {
    /// The text to display.
    #[serde(default)]
    pub text: Option<String>,
    /// The colour to use when displaying the value.
    #[serde(default)]
    pub color: Option<String>,
    /// Used for ordering in the UI.
    #[serde(default)]
    pub index: Option<isize>,
}

/// Configuration for thresholds for a field.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ThresholdsConfig {
    /// How thresholds should be evaluated.
    pub mode: ThresholdsMode,
    /// The threshold steps.
    pub steps: Vec<Threshold>,
}

/// A single step on the threshold list.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Threshold {
    /// The upper bound of this threshold.
    pub value: Option<ConfFloat64>,
    /// The colour to use for this threshold.
    pub color: Option<String>,
    /// The state of the alert.
    pub state: Option<String>,
}

/// How thresholds should be evaluated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ThresholdsMode {
    /// Pick thresholds based on the absolute value.
    ///
    /// This is the default value.
    Absolute,
    /// Thresholds should be treated as relative to the min/max.
    Percentage,
}

impl Default for ThresholdsMode {
    fn default() -> Self {
        Self::Absolute
    }
}

/// Links to use when clicking on a result.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DataLink {
    /// The title text to display.
    pub title: Option<String>,
    /// Is the target blank?
    pub target_blank: Option<bool>,
    /// The URL to link to.
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {

    use std::time::UNIX_EPOCH;

    use chrono::prelude::*;
    use paste::paste;

    use super::*;

    macro_rules! test_create_field_from_type {
        ($t:ty) => {
            paste! {
                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_ $t >]() {
                    let field = [<$t>::default()].into_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_opt_ $t >]() {
                    let field = [Some(<$t>::default()), None].into_opt_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 2)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_iter_ $t >]() {
                    let field = std::iter::once(<$t>::default()).into_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_opt_iter_ $t >]() {
                    let field = std::iter::once(Some(<$t>::default())).into_opt_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_vec_ $t >]() {
                    let array = <$t as FieldType>::InArray::from(vec![<$t>::default()]);
                    let field = array.try_into_field("x".to_string()).unwrap();
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }
            }
        };
        ($t:ty, $e: expr) => {
            paste! {
                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_ $t >]() {
                    let field = [$e].into_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_opt_ $t >]() {
                    let field = [Some($e), None].into_opt_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 2)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_iter_ $t >]() {
                    let field = std::iter::once($e).into_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }

                #[test]
                #[allow(non_snake_case)]
                fn [< create_field_from_opt_iter_ $t >]() {
                    let field = std::iter::once(Some($e)).into_opt_field("x".to_string());
                    assert_eq!(field.name, "x");
                    assert_eq!(field.values.len(), 1)
                }
            }
        };
    }

    test_create_field_from_type!(bool);
    test_create_field_from_type!(i8);
    test_create_field_from_type!(i16);
    test_create_field_from_type!(i32);
    test_create_field_from_type!(i64);
    test_create_field_from_type!(u8);
    test_create_field_from_type!(u16);
    test_create_field_from_type!(u32);
    test_create_field_from_type!(u64);
    test_create_field_from_type!(f32);
    test_create_field_from_type!(f64);
    test_create_field_from_type!(String);
    test_create_field_from_type!(str, "");
    test_create_field_from_type!(SystemTime, UNIX_EPOCH);
    test_create_field_from_type!(
        NaiveDateTime,
        NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
    test_create_field_from_type!(NaiveDate, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    test_create_field_from_type!(
        DateTime,
        Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).single().unwrap()
    );
    test_create_field_from_type!(
        Date,
        Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).single().unwrap()
    );

    #[test]
    fn set_values_from_iter_primitive() {
        let mut field = vec![1u32, 2, 3].into_field("yhat".to_string());
        assert!(field.set_values([4u32, 5, 6]).is_ok());
        assert!(field.set_values([4u64, 5, 6]).is_err());
    }

    // #[test]
    // fn set_values_from_array_primitive() {
    //     let array = PrimitiveArray::<u32>::from_slice([1u32, 2, 3]);
    //     let mut field = array.try_into_field("yhat".to_string()).unwrap();
    //     let array = PrimitiveArray::<u32>::from_slice([4u32, 5, 6]);
    //     assert!(field.set_values(array).is_ok());
    //     let array = PrimitiveArray::<u64>::from_slice([4u64, 5, 6]);
    //     assert!(field.set_values(array).is_err());
    // }
}
