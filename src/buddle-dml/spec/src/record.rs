use buddle_utils::ahash::RandomState;
use indexmap::IndexMap;
use roxmltree::Node;

use crate::field::Field;

/// Represents a DML record which groups [`Field`]s together.
///
/// Records hold an unspecified number of fields in a fixed order that must
/// be respected when encoding and decoding them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    fields: IndexMap<String, Field, RandomState>,
    pub(crate) order: u8,
}

impl Record {
    pub(crate) fn parse(node: Node<'_, '_>) -> Self {
        let mut record = Self {
            fields: IndexMap::default(),
            order: 0,
        };

        for node in node.children().filter(Node::is_element) {
            let field = Field::parse(node);
            record.fields.insert(field.name().to_string(), field);
        }

        record
    }

    /// Gets an immutable reference to a [`Field`] by its name.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.get(name)
    }

    /// Constructs an iterator over all the [`Field`]s which do not have the
    /// `NOXFER` attribute set.
    ///
    /// This guarantees the correct order of the yielded fields.
    pub fn iter_visible_fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.values().filter(|f| !f.noxfer())
    }

    /// Gets the Service ID of the protocol under the premise that this is
    /// the protocol info record.
    pub fn service_id(&self) -> Option<u8> {
        self.field("ServiceID")
            .and_then(Field::value)
            .and_then(|v| v.parse().ok())
    }

    /// Gets the protocol version under the premise that this is the protocol
    /// info record.
    pub fn protocol_type(&self) -> Option<&str> {
        self.field("ProtocolType").and_then(Field::value)
    }

    /// Gets the protocol version under the premise that this is the protocol
    /// info record.
    pub fn protocol_version(&self) -> Option<i32> {
        self.field("ProtocolVersion")
            .and_then(Field::value)
            .and_then(|v| v.parse().ok())
    }

    /// Gets the protocol description under the premise that this is the
    /// protocol info record.
    pub fn protocol_description(&self) -> Option<&str> {
        self.field("ProtocolDescription").and_then(Field::value)
    }

    /// Gets the message name of the record under the premise that it is
    /// a protocol message.
    pub fn message_name(&self) -> Option<&str> {
        self.field("_MsgName").and_then(Field::value)
    }

    /// Gets the order value of the record under the premise that it is a
    /// protocol message.
    pub fn message_order(&self) -> u8 {
        self.field("_MsgOrder")
            .and_then(Field::value)
            .and_then(|v| v.parse().ok())
            .unwrap_or(self.order)
    }

    /// Gets the description string of this record under the premise that
    /// it is a protocol message.
    pub fn message_description(&self) -> Option<&str> {
        self.field("_MsgDescription").and_then(Field::value)
    }

    /// Gets the handler callback of this record under the premise that it
    /// is a protocol message.
    pub fn message_handler(&self) -> Option<&str> {
        self.field("_MsgHandler").and_then(Field::value)
    }

    /// Gets the access level of this record under the premise that it is a
    /// protocol message.
    pub fn message_access_level(&self) -> u8 {
        self.field("_MsgAccessLvl")
            .and_then(Field::value)
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    }
}
