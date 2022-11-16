//! Entity handling types.

use buddle_math::Vec3A;
use buddle_object_property::{cpp, Type};
use buddle_utils::hash::byte_string_id;

/// The base type for any entities in the system.
///
/// This is the base representation of every object in
/// the game. They uniquely own [`BehaviorInstance`]s
/// which extend their state and functionality.
///
/// This draws inspiration from [Medusa]'s [`Noun`] class.
///
/// [Medusa]: https://github.com/palestar/medusa
/// [`Noun`]: https://github.com/palestar/medusa/blob/develop/World/Noun.h
#[derive(Default, Type)]
#[object(name = "class CoreObject")]
pub struct CoreObject {
    #[property(
        name = "m_globalID.m_full",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT | OBJECT_ID)
    )]
    pub global_id: u64, // TODO: Proper GID type.
    #[property(
        name = "m_permID",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT | OBJECT_ID)
    )]
    pub perm_id: u64, // TODO: Proper GID type.
    #[property(
        name = "m_location",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub location: Vec3A,
    #[property(
        name = "m_orientation",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub orientation: Vec3A,
    #[property(
        name = "m_fScale",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub scale: f32,
    #[property(
        name = "m_templateID.m_full",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub template_id: u64, // TODO: Proper GID type.
    #[property(
        name = "m_debugName",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub debug_name: cpp::RawString,
    #[property(
        name = "m_displayKey",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub display_key: cpp::RawString,
    #[property(
        name = "m_zoneTagID",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub zone_tag_id: u32,
    #[property(
        name = "m_speedMultiplier",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub speed_multiplier: i16,
    #[property(
        name = "m_nMobileID",
        flags(SAVE | COPY | PUBLIC | TRANSMIT | PRIVILEGED_TRANSMIT)
    )]
    pub mobile_id: u16,
}

/// A base type for behavior templates.
///
/// A behavior template encodes state that is used to
/// create an associated [`BehaviorInstance`] subclass.
///
/// As such, a [`BehaviorTemplate`] is directly convertible
/// into [`BehaviorInstance`] for type construction.
#[derive(Clone, Default, Type)]
#[object(name = "class BehaviorTemplate")]
pub struct BehaviorTemplate {
    /// The name of the behavior.
    #[property(name = "m_behaviorName", flags(SAVE | COPY | PUBLIC))]
    pub behavior_name: cpp::RawString,
}

/// The base type for core templates.
///
/// A core template directly maps to a [`CoreObject`] type.
///
/// It provides the necessary info to create the object
/// itself, but also a list of behavior templates to
/// create the behaviors that should be attached.
/// [`BehaviorTemplate`]
#[derive(Default, Type)]
#[object(name = "class CoreTemplate")]
pub struct CoreTemplate {
    /// The [`BehaviorTemplate`]s for the type.
    #[property(name = "m_behaviors", flags(SAVE | COPY | PUBLIC))]
    pub behaviors: Vec<cpp::Ptr<BehaviorTemplate>>,
}

/// The base type for any behavior instances.
///
/// Behaviors represent the **components** in the ECS and
/// are attached to [`CoreObject`]s.
///
/// They dynamically provide more state without having to
/// bloat the [`CoreObject`] types, but do not define
/// functionality by themselves.
///
/// This draws inspiration from [Medusa]'s [`Trait`] class.
///
/// [Medusa]: https://github.com/palestar/medusa
/// [`Trait`]: https://github.com/palestar/medusa/blob/develop/World/Trait.h
#[derive(Default, Type)]
#[object(name = "class BehaviorInstance")]
pub struct BehaviorInstance {
    /// The string ID of the associated [`BehaviorTemplate`]
    /// name.
    #[property(
        name = "m_behaviorTemplateNameID",
        flags(SAVE | COPY | PUBLIC | PERSIST)
    )]
    pub behavior_template_name_id: u32,
}

impl From<BehaviorTemplate> for BehaviorInstance {
    fn from(value: BehaviorTemplate) -> Self {
        Self {
            behavior_template_name_id: byte_string_id(&value.behavior_name),
        }
    }
}
