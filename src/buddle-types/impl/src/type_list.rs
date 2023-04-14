use std::{collections::HashMap, io};

use anyhow::anyhow;
use buddle_object_property::type_info::PropertyFlags;
use serde::{Deserialize, Deserializer};

/// Representation of a type list for all the embedded
/// type information in the game client.
#[derive(Clone, Deserialize)]
pub struct TypeList {
    /// Version constant of the format.
    pub version: u32,
    /// A mapping of type definitions.
    pub classes: HashMap<u32, TypeDef>,
}

impl TypeList {
    /// Deserializes a type list in JSON format from a given reader.
    pub fn from_reader<R: io::Read>(reader: R) -> anyhow::Result<Self> {
        serde_json::from_reader(reader).map_err(Into::into)
    }
}

/// An individual type definition inside the list.
#[derive(Clone, Deserialize)]
pub struct TypeDef {
    /// The base classes of a type, if any.
    pub bases: Vec<String>,
    /// The type name.
    pub name: String,
    /// The hash of the type name.
    pub hash: u32,
    /// The properties of the class.
    #[serde(deserialize_with = "deserialize_property_list")]
    pub properties: Vec<Property>,
}

/// A property that represents a member of a class.
#[derive(Clone, Deserialize)]
pub struct Property {
    /// The name of the property.
    #[serde(skip)]
    pub name: String,
    /// The type of the property.
    pub r#type: String,
    /// The ID of the property.
    pub id: u32,
    /// The offset of the property into the class.
    pub offset: usize,
    /// The associated property flag mask.
    #[serde(deserialize_with = "deserialize_property_flags")]
    pub flags: PropertyFlags,
    /// The underlying container of the property.
    pub container: String,
    /// Whether the property's storage is dynamically allocated.
    pub dynamic: bool,
    /// Whether the property's type is a global singleton.
    pub singleton: bool, // FIXME: I'm at the wrong place.
    /// Whether the property is a pointer.
    pub pointer: bool,
    /// A combined hash of the property's name and of its type name.
    pub hash: u32,
    /// A mapping of all enum options defined on a property.
    #[serde(default)]
    pub enum_options: HashMap<String, StringOrInt>,
}

/// Hack to deal with some inconsistencies in how options are stored.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    String(String),
    Int(i64),
}

fn deserialize_property_list<'de, D>(deserializer: D) -> Result<Vec<Property>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut properties: Vec<_> = HashMap::<String, Property>::deserialize(deserializer)?
        .drain()
        .map(|(name, mut property)| {
            property.name = name;
            property
        })
        .collect();

    // Sort properties by ID for correct order.
    properties.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(properties)
}

fn deserialize_property_flags<'de, D>(deserializer: D) -> Result<PropertyFlags, D::Error>
where
    D: Deserializer<'de>,
{
    PropertyFlags::from_bits(u32::deserialize(deserializer)?)
        .ok_or_else(|| serde::de::Error::custom("encountered unknown PropertyFlag bits"))
}
