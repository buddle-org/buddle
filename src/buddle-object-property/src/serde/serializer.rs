use std::io::{self, Read};

use buddle_bit_buf::BitWriter;
use byteorder::{WriteBytesExt, LE};
use flate2::{read::ZlibEncoder, Compression};

use super::{Config, SerializerFlags, TypeTag};
use crate::{property_class::PropertyClass, r#enum::Enum, type_info::PropertyFlags};

fn zlib_compress(data: &[u8], buffer: &mut Vec<u8>) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(data, Compression::default());
    encoder.read_to_end(buffer)?;
    Ok(())
}

macro_rules! impl_write_len {
    ($($fn:ident = $write_fn:ident()),* $(,)?) => {
        $(
            #[inline]
            fn $fn(&mut self, len: usize) {
                if self.config.flags.contains(SerializerFlags::COMPACT_LENGTH_PREFIXES) {
                    self.write_compact_length_prefix(len);
                } else {
                    self.writer.$write_fn(len as _);
                }
            }
        )*
    };
}

/// A binary serializer for objects in the *ObjectProperty* system.
///
/// This is the only officially supported serialization mechanism.
pub struct Serializer<'a> {
    writer: BitWriter,
    config: Config,
    tag: &'a dyn TypeTag,
}

impl<'a> Serializer<'a> {
    /// Creates a new serializer from the given [`Config`].
    pub fn new(config: Config, tag: &'a dyn TypeTag) -> Self {
        let mut this = Self {
            writer: BitWriter::new(),
            config,
            tag,
        };

        // As an optimization, we can write the flags directly if we
        // do not have to compress afterwards. This saves quite a few
        // memory allocations most of the time.
        let flags = this.config.flags;
        if flags.contains(SerializerFlags::STATEFUL_FLAGS)
            && !flags.contains(SerializerFlags::COMPRESS)
        {
            this.writer.u32(flags.bits() as u32);
        }

        this
    }

    /// Finishes serialization and returns the raw state.
    pub fn finish(self) -> io::Result<Vec<u8>> {
        let flags = self.config.flags;

        // When we don't have to compress, we do not have to allocate
        // more memory to finish the serialization.
        if !flags.contains(SerializerFlags::COMPRESS) {
            return Ok(self.writer.into_vec());
        }

        // We have to handle compression, so prepare a new buffer.
        let mut output = Vec::new();
        let state = self.writer.view();

        // Store the configuration flags, if necessary.
        if flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            output.write_u32::<LE>(flags.bits() as u32)?;
        }

        // While KI technically supports sending decompressed state when
        // it happens to be smaller than its compressed equivalent, we
        // do not handle this behavior because it rarely happens in practice.
        output.push(1);
        output.write_u32::<LE>(state.len() as u32)?;
        zlib_compress(state, &mut output)?;

        Ok(output)
    }

    /// Finishes serialization and returns the raw, compressed state.
    ///
    /// NOTE: Do not use this method when [`SerializerFlags::COMPRESS`]
    /// was configured for serialization.
    pub fn finish_compressed(self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let state = self.writer.view();

        output.write_u32::<LE>(state.len() as u32)?;
        zlib_compress(state, &mut output)?;

        Ok(output)
    }

    /// Serializes a [`PropertyClass`] object to the stream.
    ///
    /// This should be used as the entrypoint to serialization.
    ///
    /// Serialization is infallible, so this will always succeed.
    pub fn serialize(&mut self, v: &mut dyn PropertyClass) {
        self.try_serialize(Some(v));
    }

    /// Serializes a [`PropertyClass`] object to the stream, if any.
    ///
    /// Serialization is infallible, so this will always succeed.
    pub fn try_serialize(&mut self, v: Option<&mut dyn PropertyClass>) {
        // Write the object's type tag.
        self.tag.write_tag(self, v.as_deref());

        // Serialize either an object if we got one, or bail.
        let obj = match v {
            Some(v) => v,
            None => return,
        };

        obj.on_pre_save();

        if self.config.shallow {
            self.serialize_properties_shallow(obj);
        } else {
            // Reserve a placeholder for the object's size in bits.
            let object_size = self.writer.reserve_length_prefix::<u32>();

            // Serialize the object itself.
            self.serialize_properties_deep(obj);

            // Patch back the object size.
            self.writer.write_length_prefix(object_size);
        }

        obj.on_post_save();
    }

    /// Provides access to the underlying [`BitWriter`].
    #[inline]
    pub fn writer(&mut self) -> &mut BitWriter {
        &mut self.writer
    }

    fn write_compact_length_prefix(&mut self, len: usize) {
        if len <= u8::MAX as usize >> 1 {
            self.writer.write_bit(false);
            self.writer.write_bitint(len, u8::BITS as usize - 1);
        } else {
            self.writer.write_bit(true);
            self.writer.write_bitint(len, u32::BITS as usize - 1);
        }
    }

    impl_write_len! {
        // Used for strings, where the length is written as `u16`.
        write_str_len = u16(),

        // Used for collections, where the length is written as `u32`.
        write_seq_len = u32(),
    }

    /// Serializes the raw data of a string, including the length prefix.
    pub fn write_str(&mut self, data: &[u8]) {
        self.write_str_len(data.len());
        self.writer.write_bytes(data);
    }

    /// Serializes the raw data of a wide string, including the length prefix.
    pub fn write_wstr(&mut self, data: &[u16]) {
        self.write_str_len(data.len());
        data.iter().copied().for_each(|c| self.writer.u16(c));
    }

    fn serialize_properties_shallow(&mut self, v: &mut dyn PropertyClass) {
        // If this object has a base type, we will serialize its properties
        // without a dedicated header as if they were part of this object.
        if let Some(base) = v.base_mut() {
            self.serialize_properties_shallow(base);
        }

        let list = v.property_list();
        let mask = self.config.property_mask;

        // Iterate through all masked properties and serialize their values.
        for property in list.iter_properties().filter(|p| {
            let flags = p.flags();
            flags.contains(mask) && !flags.contains(PropertyFlags::DEPRECATED)
        }) {
            v.property_mut(property).serialize(self);
        }
    }

    fn serialize_properties_deep(&mut self, v: &mut dyn PropertyClass) {
        // If this object has a base type, we will serialize its properties
        // without a header as if they were part of this object.
        if let Some(base) = v.base_mut() {
            self.serialize_properties_deep(base);
        }

        for property in v.property_list().iter_properties() {
            // Reserve a placeholder for the property's size.
            let property_size = self.writer.reserve_length_prefix::<u32>();

            // Write the property hash.
            self.writer.u32(property.hash());

            // Serialize the property's value.
            v.property_mut(property).serialize(self);

            // Patch back the property size.
            self.writer.write_length_prefix(property_size);
        }
    }

    /// Serializes the length of a [`Container`][crate::Container] object
    /// to the stream, followed by the elements themselves.
    ///
    /// This method may be used to implement
    /// [`Type::serialize`][crate::Type::serialize] for containers.
    pub fn serialize_container_len(&mut self, len: usize) {
        self.write_seq_len(len);
    }

    /// Serializes an [`Enum`] object to the stream.
    ///
    /// This method may be used to implement
    /// [`Type::serialize`][crate::Type::serialize] for enums.
    pub fn serialize_enum(&mut self, v: &dyn Enum) {
        if self
            .config
            .flags
            .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
        {
            self.write_str(v.variant().as_bytes());
        } else {
            self.writer.u32(v.value());
        }
    }
}
