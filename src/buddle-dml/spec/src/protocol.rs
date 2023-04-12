use std::cmp::Ordering;

use anyhow::{anyhow, bail};
use buddle_utils::ahash::RandomState;
use indexmap::IndexMap;
use roxmltree::{Document, Node};

use crate::record::Record;

const PROTOCOL_INFO: &str = "_ProtocolInfo";

/// A DML protocol storing message [`Record`]s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Protocol {
    name: String,
    records: IndexMap<String, Record, RandomState>,
}

impl Protocol {
    pub(crate) fn parse(document: Document<'_>) -> anyhow::Result<Self> {
        let root = document.root().children().next().unwrap();
        let mut proto = Self {
            name: root.tag_name().name().to_string(),
            records: IndexMap::default(),
        };

        for record in root.children().filter(Node::is_element) {
            let tag = record.tag_name().name();
            let record = record
                .children()
                .find(|c| c.has_tag_name("RECORD"))
                .ok_or_else(|| anyhow!("cannot find RECORD node in protocol entry"))?;

            proto.records.insert(tag.to_string(), Record::parse(record));
        }

        if proto.records.len() > u8::MAX as _ {
            bail!("Too many records in protocol!");
        }

        proto.sort_records()?;

        Ok(proto)
    }

    /// Gets an immutable reference to the protocol's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets an immutable reference to a protocol [`Record`] by name.
    pub fn record(&self, name: &str) -> Option<&Record> {
        self.records.get(name)
    }

    /// Gets an immutable reference to the protocol's info record, if exists.
    pub fn protocol_info(&self) -> Option<&Record> {
        self.record(PROTOCOL_INFO)
    }

    /// Gets an iterator over all the [`Record`]s in the protocol along with
    /// their names.
    pub fn iter_records(&self) -> impl Iterator<Item = (&str, &Record)> {
        self.records.iter().map(|(n, r)| (n.as_str(), r))
    }

    /// Gets an iterator over only the message [`Record`]s in the protocol
    /// along with their names.
    pub fn iter_messages(&self) -> impl Iterator<Item = (&str, &Record)> {
        self.iter_records().filter(|(n, _)| *n != PROTOCOL_INFO)
    }

    fn sort_records(&mut self) -> anyhow::Result<()> {
        // Check if all the records already have order values assigned.
        if self
            .records
            .iter()
            .all(|(n, r)| n == PROTOCOL_INFO || r.message_order() != 0)
        {
            return Ok(());
        }

        // Next, we check that none of the records have order values assigned.
        // In that case we can manually sort them and assign our owns.
        if self
            .records
            .iter()
            .all(|(n, r)| n == PROTOCOL_INFO || r.message_order() == 0)
        {
            // First, sort the entries in the map by order values.
            // `_ProtocolInfo` always gets the order value of 0.
            self.records
                .sort_by(|ak, _, bk, _| match (ak.as_str(), bk.as_str()) {
                    (PROTOCOL_INFO, _) => Ordering::Less,
                    (_, PROTOCOL_INFO) => Ordering::Greater,
                    (ak, bk) => ak.cmp(bk),
                });

            // Then, assign the indices of all the entries as the order values
            // of the records in it.
            self.records
                .iter_mut()
                .enumerate()
                .for_each(|(idx, (_, r))| r.order = idx as u8);

            return Ok(());
        }

        // When we land here, some elements have order values and others don't.
        // This kind of mixup is often not correctable so we leave it to the
        // protocol author to come up with a consistent style.
        bail!("If explicit `_MsgOrder`s are specified, do so on all or no messages");
    }
}
