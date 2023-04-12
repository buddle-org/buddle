use binrw::{binread, BinRead};

use super::*;
use crate::{basic::*, enums::*, parse};

/// A string of given length.
#[derive(Clone, Debug, Default, PartialEq, BinRead)]
pub struct SizedString {
    #[br(parse_with = parse::sized_string)]
    pub data: String,
}

/// A string of given length.
#[derive(Clone, Debug, Default, PartialEq, BinRead)]
pub struct SizedString16 {
    #[br(parse_with = parse::sized_string_16)]
    pub data: String,
}

/// A generic key with support for interpolation. Type 1 is normal linear interpolation, type 2 has forward and backward tangents, and type 3 has tension, bias and continuity arguments. Note that color4 and byte always seem to be of type 1.
#[derive(Clone, Debug, PartialEq, BinRead)]
#[br(import(interpolation: Option < KeyType >))]
pub struct Key<T> where for<'a> T: BinRead<Args<'a>=()>, {
    pub time: f32,
    pub value: T,
    #[br(if (interpolation == Some(KeyType::QUADRATIC_KEY)))]
    pub forward: Option<T>,
    #[br(if (interpolation == Some(KeyType::QUADRATIC_KEY)))]
    pub backward: Option<T>,
    #[br(if (interpolation == Some(KeyType::TBC_KEY)))]
    pub tbc: Option<TBC>,
}

/// Array of vector keys (anything that can be interpolated, except rotations).
#[binread]
#[derive(Clone, Debug, PartialEq)]
pub struct KeyGroup<T>
    where
            for<'a> T: BinRead<Args<'a>=()> + 'a, {
    #[br(temp)]
    num_keys: u32,
    #[br(if (num_keys != 0))]
    pub interpolation: Option<KeyType>,
    #[br(args { count: num_keys as usize, inner: (interpolation,) })]
    pub keys: Vec<Key<T>>,
}

#[derive(Clone, Debug, PartialEq, BinRead)]
pub struct BoundingVolume {
    pub collision_type: BoundVolumeType,
    #[br(if (collision_type == BoundVolumeType::SPHERE_BV))]
    pub sphere: Option<NiBound>,
    #[br(if (collision_type == BoundVolumeType::BOX_BV))]
    pub r#box: Option<BoxBV>,
    #[br(if (collision_type == BoundVolumeType::CAPSULE_BV))]
    pub capsule: Option<CapsuleBV>,
    #[br(if (collision_type == BoundVolumeType::HALFSPACE_BV))]
    pub half_space: Option<HalfSpaceBV>,
}
