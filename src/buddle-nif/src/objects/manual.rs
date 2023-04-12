use binrw::{binread, BinRead};
use bitflags::bitflags;

use crate::{basic::*, parse};
use binrw::{
    io::{Read, Seek},
    BinResult, Error, Endian,
};

use super::NiDataStream;
use crate::{bitfields::*, bitflags::*, compounds::*, enums::*};

#[binread]
#[derive(Clone, Debug, PartialEq)]
#[br(import(usage: u32, access: u32))]
pub(super) struct NiDataStreamTheSadWay {
    #[br(calc = (usage as usize).try_into().unwrap())]
    pub usage: DataStreamUsage,
    #[br(calc = DataStreamAccess::from_bits_truncate(access))]
    pub access: DataStreamAccess,
    #[br(temp)]
    num_bytes: u32,
    pub cloning_behavior: CloningBehavior,
    #[br(temp)]
    num_regions: u32,
    #[br(count = num_regions)]
    pub regions: Vec<Region>,
    #[br(temp)]
    num_components: u32,
    #[br(count = num_components)]
    pub component_formats: Vec<ComponentFormat>,
    #[br(count = num_bytes)]
    pub data: Vec<u8>,
    #[br(map = |b: u8| b != 0)]
    pub streamable: bool,
}

impl From<NiDataStreamTheSadWay> for NiDataStream {
    fn from(value: NiDataStreamTheSadWay) -> Self {
        Self {
            usage: value.usage,
            access: value.access,
            cloning_behavior: value.cloning_behavior,
            regions: value.regions,
            component_formats: value.component_formats,
            data: value.data,
            streamable: value.streamable,
        }
    }
}
