use roxmltree::Node;

/// Representation of a DML field inside a record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    name: String,
    value: Option<String>,

    r#type: Option<String>,
    noxfer: bool,
}

impl Field {
    pub(crate) fn parse(node: Node<'_, '_>) -> Self {
        Self {
            name: node.tag_name().name().to_string(),
            value: node.text().map(String::from),

            r#type: node.attribute("TYPE").map(String::from),
            noxfer: node
                .attribute("NOXFER")
                .map(|key| key == "TRUE")
                .unwrap_or(false),
        }
    }

    /// Gets an immutable reference to the field's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets an immutable reference to the field's value, if it has one.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// Gets the DML type of the field if it has one assigned.
    pub fn dml_type(&self) -> Option<&str> {
        self.r#type.as_deref()
    }

    /// Indicates if this field should be transferred over the network.
    pub fn noxfer(&self) -> bool {
        self.noxfer
    }
}
