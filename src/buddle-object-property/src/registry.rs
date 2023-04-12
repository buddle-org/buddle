use std::collections::HashMap;


use crate::serde::TypeTag;
use crate::type_info::{PropertyList, Reflected, TypeInfo::Class};

pub struct Registry {
    registry: HashMap<u32, &'static PropertyList>
}

impl Registry {
    pub fn register<T: Reflected>(&mut self) {
        match T::TYPE_INFO {
            Class(list) => {
                self.registry.insert(list.type_hash(), list);
            }
            _ => panic!("Expected Class not leaf")
        }
    }
}

impl TypeTag for Registry {
    fn read_tag(&self, de: &mut crate::serde::Deserializer<'_>)
        -> anyhow::Result<Option<Box<dyn crate::PropertyClass>>> {
        let type_hash = de.reader().u32()?;
        
        if type_hash == 0 {
            return Ok(None);
        }

        let list = self.registry.get(&type_hash).ok_or_else(|| anyhow::anyhow!("Hash {type_hash} not in registry"))?;
        Ok(Some(list.make_default()))
    }

    fn validate_tag(
        &self,
        de: &mut crate::serde::Deserializer<'_>,
        obj: &dyn crate::PropertyClass,
    ) -> anyhow::Result<()> {
        let type_hash = de.reader().u32()?;

        if type_hash == obj.property_list().type_hash() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Hashes don't match"))
        }
    }

    fn write_tag(&self, ser: &mut crate::serde::Serializer<'_>, obj: Option<&dyn crate::PropertyClass>) {
        let type_hash = match obj {
            Some(class) => class.property_list().type_hash(),
            None => 0
        };

        ser.writer().u32(type_hash);

        //small rust version for vale
        //ser.writer().u32(obj.map_or(0, |class| class.property_list().type_hash()));
    }
}
