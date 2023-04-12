// This file is auto-generated by nifgen
// based on nifxml version 0.9.3.0.
// Do not edit manually.

#![allow(
    clippy::eq_op,
    clippy::identity_op,
    non_camel_case_types,
    non_upper_case_globals,
    unused_imports,
    unused_parens
)]

use binrw::BinRead;
use bitflags::bitflags;

use crate::{basic::*, parse};

bitflags! {
    /// Describes the options for the accum root on NiControllerSequence.
    #[derive(BinRead)]
    pub struct AccumFlags: u32 {
        const ACCUM_X_TRANS = 0;
        const ACCUM_Y_TRANS = 1;
        const ACCUM_Z_TRANS = 2;
        const ACCUM_X_ROT = 3;
        const ACCUM_Y_ROT = 4;
        const ACCUM_Z_ROT = 5;
        const ACCUM_X_FRONT = 6;
        const ACCUM_Y_FRONT = 7;
        const ACCUM_Z_FRONT = 8;
        const ACCUM_NEG_FRONT = 9;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct PathFlags: u16 {
        const NIPI_CVDataNeedsUpdate = 0;
        const NIPI_CurveTypeOpen = 1;
        const NIPI_AllowFlip = 2;
        const NIPI_Bank = 3;
        const NIPI_ConstantVelocity = 4;
        const NIPI_Follow = 5;
        const NIPI_Flip = 6;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct LookAtFlags: u16 {
        const LOOK_FLIP = 0;
        const LOOK_Y_AXIS = 1;
        const LOOK_Z_AXIS = 2;
    }
}

bitflags! {
    /// Flags for NiSwitchNode.
    #[derive(BinRead)]
    pub struct NiSwitchFlags: u16 {
        const UpdateOnlyActiveChild = 0;
        const UpdateControllers = 1;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct NxBodyFlag: u32 {
        const NX_BF_DISABLE_GRAVITY = 0;
        const NX_BF_FROZEN_POS_X = 1;
        const NX_BF_FROZEN_POS_Y = 2;
        const NX_BF_FROZEN_POS_Z = 3;
        const NX_BF_FROZEN_ROT_X = 4;
        const NX_BF_FROZEN_ROT_Y = 5;
        const NX_BF_FROZEN_ROT_Z = 6;
        const NX_BF_KINEMATIC = 7;
        const NX_BF_VISUALIZATION = 8;
        const NX_BF_POSE_SLEEP_TEST = 9;
        const NX_BF_FILTER_SLEEP_VEL = 10;
        const NX_BF_ENERGY_SLEEP_TEST = 11;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct NxShapeFlag: u32 {
        const NX_SF_TRIGGER_ON_ENTER = 0;
        const NX_SF_TRIGGER_ON_LEAVE = 1;
        const NX_SF_TRIGGER_ON_STAY = 2;
        const NX_SF_VISUALIZATION = 3;
        const NX_SF_DISABLE_COLLISION = 4;
        const NX_SF_FEATURE_INDICES = 5;
        const NX_SF_DISABLE_RAYCASTING = 6;
        const NX_SF_POINT_CONTACT_FORCE = 7;
        const NX_SF_FLUID_DRAIN = 8;
        const NX_SF_FLUID_DISABLE_COLLISION = 10;
        const NX_SF_FLUID_TWOWAY = 11;
        const NX_SF_DISABLE_RESPONSE = 12;
        const NX_SF_DYNAMIC_DYNAMIC_CCD = 13;
        const NX_SF_DISABLE_SCENE_QUERIES = 14;
        const NX_SF_CLOTH_DRAIN = 15;
        const NX_SF_CLOTH_DISABLE_COLLISION = 16;
        const NX_SF_CLOTH_TWOWAY = 17;
        const NX_SF_SOFTBODY_DRAIN = 18;
        const NX_SF_SOFTBODY_DISABLE_COLLISION = 19;
        const NX_SF_SOFTBODY_TWOWAY = 20;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct NxMaterialFlag: u32 {
        const NX_MF_ANISOTROPIC = 0;
        const NX_MF_DISABLE_FRICTION = 4;
        const NX_MF_DISABLE_STRONG_FRICTION = 5;
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct NxClothFlag: u32 {
        const NX_CLF_PRESSURE = 0;
        const NX_CLF_STATIC = 1;
        const NX_CLF_DISABLE_COLLISION = 2;
        const NX_CLF_SELFCOLLISION = 3;
        const NX_CLF_VISUALIZATION = 4;
        const NX_CLF_GRAVITY = 5;
        const NX_CLF_BENDING = 6;
        const NX_CLF_BENDING_ORTHO = 7;
        const NX_CLF_DAMPING = 8;
        const NX_CLF_COLLISION_TWOWAY = 9;
        const NX_CLF_TRIANGLE_COLLISION = 11;
        const NX_CLF_TEARABLE = 12;
        const NX_CLF_HARDWARE = 13;
        const NX_CLF_COMDAMPING = 14;
        const NX_CLF_VALIDBOUNDS = 15;
        const NX_CLF_FLUID_COLLISION = 16;
        const NX_CLF_DISABLE_DYNAMIC_CCD = 17;
        const NX_CLF_ADHERE = 18;
    }
}

bitflags! {
    /// Determines how the data stream is accessed?
    #[derive(BinRead)]
    pub struct DataStreamAccess: u32 {
        const CPURead = 0;
        const CPUWriteStatic = 1;
        const CPUWriteMutable = 2;
        const CPUWriteVolatile = 3;
        const GPURead = 4;
        const GPUWrite = 5;
        const CPUWriteStaticInititialized = 6;
    }
}

bitflags! {
    /// Flags for NiShadowGenerator.
    /// Bit Patterns:
    /// AUTO_CALC_NEARFAR = (AUTO_NEAR_DIST | AUTO_FAR_DIST) = 0xC0
    /// AUTO_CALC_FULL = (AUTO_NEAR_DIST | AUTO_FAR_DIST | AUTO_DIR_LIGHT_FRUSTUM_WIDTH | AUTO_DIR_LIGHT_FRUSTUM_POSITION) = 0x3C0
    #[derive(BinRead)]
    pub struct NiShadowGeneratorFlags: u16 {
        const DIRTY_SHADOWMAP = 0;
        const DIRTY_RENDERVIEWS = 1;
        const GEN_STATIC = 2;
        const GEN_ACTIVE = 3;
        const RENDER_BACKFACES = 4;
        const STRICTLY_OBSERVE_SIZE_HINT = 5;
        const AUTO_NEAR_DIST = 6;
        const AUTO_FAR_DIST = 7;
        const AUTO_DIR_LIGHT_FRUSTUM_WIDTH = 8;
        const AUTO_DIR_LIGHT_FRUSTUM_POSITION = 9;
    }
}
