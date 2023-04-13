/// Access control for executing incoming DML requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessLevel {
    /// No special requirements.
    None,
    /// Requires a valid session to be stablished.
    #[default]
    SessionEstablished,
    /// Custom user-defined level values starting from `2`.
    ///
    /// Higher means more privileged.
    Custom(u8),
}

impl AccessLevel {
    /// Converts an integer value into an [`AccessLevel`].
    pub const fn from_value(level: u8) -> Self {
        match level {
            0 => AccessLevel::None,
            1 => AccessLevel::SessionEstablished,
            l => AccessLevel::Custom(l),
        }
    }

    /// Consumes the level into its raw integer value.
    pub const fn into_value(self) -> u8 {
        match self {
            AccessLevel::None => 0,
            AccessLevel::SessionEstablished => 1,
            AccessLevel::Custom(l) => l,
        }
    }

    /// Indicates whether `self` grants sufficient privileges for `other`.
    pub const fn meets_level(self, other: AccessLevel) -> bool {
        other.into_value() <= self.into_value()
    }
}

impl From<u8> for AccessLevel {
    fn from(l: u8) -> Self {
        Self::from_value(l)
    }
}

impl From<AccessLevel> for u8 {
    fn from(l: AccessLevel) -> Self {
        l.into_value()
    }
}
