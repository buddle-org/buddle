/// Implementation of Euler angles.
// https://github.com/palestar/medusa/blob/develop/Math/Euler.h
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Euler {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl Euler {
    /// Creates a new Euler value given the
    /// axis angles.
    pub const fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }
}

// TODO: Finish this when needed.
