use std::{num::ParseIntError, str::FromStr};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
/// Effect flags for a control point.
pub struct EffectFlags(i32);

impl EffectFlags {
    pub const NONE: i32 = 0;
    pub const KIAI: i32 = 1 << 0;
    pub const OMIT_FIRST_BAR_LINE: i32 = 1 << 3;

    /// Check whether any of the given bitflags are set.
    pub const fn has_flag(self, flag: i32) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<EffectFlags> for i32 {
    fn from(kind: EffectFlags) -> Self {
        kind.0
    }
}

impl From<i32> for EffectFlags {
    fn from(flags: i32) -> Self {
        Self(flags)
    }
}

impl FromStr for EffectFlags {
    type Err = ParseEffectFlagsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self).map_err(ParseEffectFlagsError)
    }
}

thiserror! {
    #[error("invalid effect flags")]
    /// Error when failing to parse [`EffectFlags`].
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ParseEffectFlagsError(ParseIntError);
}
