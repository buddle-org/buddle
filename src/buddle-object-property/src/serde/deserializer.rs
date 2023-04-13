use std::io::{self, Write};

use anyhow::{anyhow, bail};
use buddle_bit_buf::BitReader;
use byteorder::{ReadBytesExt, LE};
use flate2::write::ZlibDecoder;

use crate::{property_class::PropertyClass, r#enum::Enum, type_info::PropertyFlags};

use super::{Config, SerializerFlags, TypeTag};

#[inline]
fn zlib_decompress<'a>(data: &[u8], buf: &'a mut Vec<u8>) -> io::Result<&'a [u8]> {
    let mut decoder = ZlibDecoder::new(buf);
    decoder.write_all(data)?;
    decoder.finish().map(|out| &out[..])
}

macro_rules! impl_read_len {
    ($($fn:ident = $read_fn:ident()),* $(,)?) => {
        $(
            #[inline]
            fn $fn(&mut self) -> anyhow::Result<usize> {
                if self.config.flags.contains(SerializerFlags::COMPACT_LENGTH_PREFIXES) {
                    self.read_compact_length_prefix()
                } else {
                    self.reader.$read_fn().map(|v| v as usize)
                }
            }
        )*
    };
}

/// A binary deserializer for objects in the *ObjectProperty* system.
///
/// This is the only officially supported deserialization mechanism.
pub struct Deserializer<'de> {
    reader: BitReader<'de>,
    config: Config,
    tag: &'de dyn TypeTag,
}

impl<'de> Deserializer<'de> {
    /// Creates a new deserializer from the given [`Config`].
    pub fn new(config: Config, tag: &'de dyn TypeTag) -> Self {
        Self {
            reader: BitReader::default(),
            config,
            tag,
        }
    }

    /// Provides access to the underlying [`BitReader`].
    #[inline]
    pub fn reader(&mut self) -> &mut BitReader<'de> {
        &mut self.reader
    }

    /// Employs the deserializer's recursion limit when invoking a
    /// potentially dangerous function.
    ///
    /// This will return the result of the closure passed to it.
    #[inline]
    pub fn with_recursion_limit<F, R>(&mut self, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut Self) -> anyhow::Result<R>,
    {
        self.config.recursion_limit -= 1;
        if self.config.recursion_limit == 0 {
            bail!("deserializer recursion limit exhausted");
        }

        let res = f(self);

        self.config.recursion_limit += 1;

        res
    }

    fn decompress(mut data: &[u8], scratch: &'de mut Vec<u8>) -> anyhow::Result<&'de [u8]> {
        // Read the expected decompressed size of the blob.
        let size = data.read_u32::<LE>()? as usize;

        // Clear the scratch buffer and reserve the memory in advance.
        scratch.clear();
        scratch.reserve(size);

        // Decompress the data.
        let decompressed = zlib_decompress(data, scratch)?;

        // Validate the size expectations.
        if decompressed.len() != size {
            bail!("size mismatch between compressed and uncompressed data");
        }

        Ok(decompressed)
    }

    /// Decompresses `data` into `scratch` and loads the resulting data into
    /// the deserializer.
    ///
    /// This will also do the necessary deserializer configuration upfront, so
    /// the object is ready to deserialize data after calling this method.
    pub fn decompress_and_load(
        &mut self,
        data: &[u8],
        scratch: &'de mut Vec<u8>,
    ) -> anyhow::Result<()> {
        let mut decompressed = Self::decompress(data, scratch)?;

        // If configuration flags are stateful, load them.
        if self.config.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            let raw = decompressed.read_u32::<LE>()? as u8;
            self.config.flags = SerializerFlags::from_bits_truncate(raw);
        }

        self.reader = BitReader::new(decompressed);

        Ok(())
    }

    /// Loads the given `data` into the deserializer.
    ///
    /// `scratch` is a buffer for optional decompression when necessary. This
    /// allows users to re-use scratch memory allocations for multiple objects.
    ///
    /// This will also do the necessary deserializer configuration upfront, so
    /// the object is ready to deserialize data after calling this method.
    pub fn load(&mut self, mut data: &'de [u8], scratch: &'de mut Vec<u8>) -> anyhow::Result<()> {
        // If configuration flags are stateful, load them.
        if self.config.flags.contains(SerializerFlags::STATEFUL_FLAGS) {
            let raw = data.read_u32::<LE>()? as u8;
            self.config.flags = SerializerFlags::from_bits_truncate(raw);
        }

        // Determine whether the data is compressed or not.
        if self.config.flags.contains(SerializerFlags::COMPRESS) && data.read_u8()? != 0 {
            self.reader = Self::decompress(data, scratch).map(BitReader::new)?;
        } else {
            self.reader = BitReader::new(data);
        }

        Ok(())
    }

    /// Deserializes a [`PropertyClass`] object from a loaded data
    /// stream.
    ///
    /// Empty objects are treated as errors. If this is undesired,
    /// consider [`Deserializer::try_deserialize`] instead.
    pub fn deserialize(&mut self) -> anyhow::Result<Box<dyn PropertyClass>> {
        match self.try_deserialize() {
            Ok(Some(obj)) => Ok(obj),
            Ok(None) => Err(anyhow!("received empty object")),
            Err(e) => Err(e),
        }
    }

    /// Tries to deserialize a [`PropertyClass`] object from a loaded
    /// data stream.
    ///
    /// This returns [`Ok`]`(`[`None`]`)` when no object was serialized.
    pub fn try_deserialize(&mut self) -> anyhow::Result<Option<Box<dyn PropertyClass>>> {
        self.with_recursion_limit(|ser| {
            // Read the object's type tag and make sure it belongs to `v`.
            let mut obj = match ser.tag.read_tag(ser)? {
                Some(obj) => obj,
                None => return Ok(None),
            };

            obj.on_pre_load();

            if ser.config.shallow {
                ser.deserialize_properties_shallow(&mut *obj)?;
            } else {
                // Read the object's size in bits.
                // We also back up the remaining length of the object for later validation.
                let object_size = (ser.reader.u32()? - u32::BITS) as usize;
                let remaining = ser.reader.len();

                if ser.deserialize_properties_deep(&mut *obj, object_size)? != 0 {
                    bail!("serialized object was not fully consumed");
                }

                // Validate the size expectations for the object.
                if remaining - ser.reader.len() != object_size {
                    bail!("size mismatch for serialized object");
                }
            }

            obj.on_post_load();

            Ok(Some(obj))
        })
    }

    fn read_compact_length_prefix(&mut self) -> anyhow::Result<usize> {
        let is_large = self.reader.read_bit()?;
        if is_large {
            self.reader.read_bitint(u32::BITS as usize - 1)
        } else {
            self.reader.read_bitint(u8::BITS as usize - 1)
        }
    }

    impl_read_len! {
        // Used for strings, where the length is written as `u16`.
        read_str_len = u16(),

        // Used for collections, where the length is written as `u32`.
        read_seq_len = u32(),
    }

    /// Reads raw bytes, including the length prefix.
    pub fn read_bytes(&mut self) -> anyhow::Result<&'de [u8]> {
        self.read_str_len()
            .and_then(|len| self.reader.read_bytes(len))
    }

    /// Reads the raw data of a string, including its length prefix.
    pub fn read_str(&mut self) -> anyhow::Result<Vec<u8>> {
        self.read_bytes().map(|b| b.to_vec())
    }

    /// Reads the raw data of a wide string, including its length prefix.
    pub fn read_wstr(&mut self) -> anyhow::Result<Vec<u16>> {
        self.read_str_len().and_then(|len| {
            let buf = self.reader.read_bytes(len * 2)?;

            let mut data = Vec::with_capacity(len);
            buf.chunks_exact(std::mem::size_of::<u16>()).for_each(|c| {
                // SAFETY: `.chunks_exact()` guarantees slices of correct length.
                let c: [u8; 2] = unsafe { c.try_into().unwrap_unchecked() };
                data.push(u16::from_le_bytes(c));
            });

            Ok(data)
        })
    }

    fn deserialize_properties_shallow(&mut self, v: &mut dyn PropertyClass) -> anyhow::Result<()> {
        // If we have a base type, recursively deserialize its properties first
        // as if they were part of this object, i.e. without a type tag.
        if let Some(base) = v.base_mut() {
            self.deserialize_properties_shallow(base)?;
        }

        let list = v.property_list();
        let mask = self.config.property_mask;

        // Iterate over the masked properties and deserialize their values in order.
        for property in list.iter_properties().filter(|p| {
            let flags = p.flags();
            flags.contains(mask) && !flags.contains(PropertyFlags::DEPRECATED)
        }) {
            v.property_mut(property).deserialize(self)?;
        }

        Ok(())
    }

    fn deserialize_properties_deep(
        &mut self,
        v: &mut dyn PropertyClass,
        mut size: usize,
    ) -> anyhow::Result<usize> {
        // If this object has a base type, we will deserialize its properties
        // without a header as if they were part of this object.
        if let Some(base) = v.base_mut() {
            size = self.deserialize_properties_deep(base, size)?;
        }

        // Consume data until the object size drops to 0.
        for property in v.property_list().iter_properties() {
            // Back up the current buffer length and read the next property's size.
            // This will also count padding bits towards byte boundary.
            let property_start = self.reader.len();
            let property_size = self.reader.u32()? as usize;

            // Read the property's hash and find the associated entry.
            let property_hash = self.reader.u32()?;
            if property.hash() != property_hash {
                bail!("received unknown property hash {property_hash}");
            }

            // Deserialize the property's value.
            v.property_mut(property).deserialize(self)?;

            // Validate the property's size.
            if property_start - self.reader.len() != property_size {
                bail!("received property with invalid length");
            }

            size = size.checked_sub(property_size).ok_or_else(|| {
                anyhow!("encoded object size does not match sum of property sizes")
            })?;
        }

        Ok(size)
    }

    /// Deserializes a [`PropertyClass`] object from the stream in-place.
    ///
    /// This method may be used to implement [`Type::deserialize`] for classes.
    /// Do not use it as the generic entrypoint to type deserialization.
    ///
    /// [`Type::deserialize`]: crate::Type::deserialize
    pub fn deserialize_class(&mut self, v: &mut dyn PropertyClass) -> anyhow::Result<()> {
        self.with_recursion_limit(|ser| {
            // Read the object's type tag and make sure it belongs to `v`.
            ser.tag.validate_tag(ser, v)?;

            v.on_pre_load();

            if ser.config.shallow {
                ser.deserialize_properties_shallow(v)?;
            } else {
                // Read the object's size in bits.
                // We also back up the remaining length of the object for later validation.
                let object_size = (ser.reader.u32()? - u32::BITS) as usize;
                let remaining = ser.reader.len();

                if ser.deserialize_properties_deep(v, object_size)? != 0 {
                    bail!("serialized object was not fully consumed");
                }

                // Validate the size expectations for the object.
                if remaining - ser.reader.len() != object_size {
                    bail!("size mismatch for serialized object");
                }
            }

            v.on_post_load();

            Ok(())
        })
    }

    /// Reads the length of a [`Container`], denoting the number of elements
    /// that can be read afterwards.
    ///
    /// This method may be used to implement [`Type::deserialize`] for
    /// containers.
    ///
    /// [`Container`]: crate::container::Container
    /// [`Type::deserialize`]: crate::Type::deserialize
    #[inline]
    pub fn deserialize_container_len(&mut self) -> anyhow::Result<usize> {
        self.read_seq_len()
    }

    /// Deserializes an [`Enum`] variant in-place.
    ///
    /// This method may be used to implement [`Type::deserialize`] for
    /// enums.
    ///
    /// [`Type::deserialize`]: crate::Type::deserialize
    pub fn deserialize_enum(&mut self, v: &mut dyn Enum) -> anyhow::Result<()> {
        let success = if self
            .config
            .flags
            .contains(SerializerFlags::HUMAN_READABLE_ENUMS)
        {
            let variant = std::str::from_utf8(self.read_bytes()?)?;
            v.update_variant(variant)
        } else {
            let variant = self.reader.u32()?;
            v.update_value(variant)
        };

        if !success {
            bail!("invalid variant encountered for enum");
        }

        Ok(())
    }
}
