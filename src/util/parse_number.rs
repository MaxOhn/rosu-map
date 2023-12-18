use std::num;

/// The default limit when parsing via [`ParseNumber`].
pub const MAX_PARSE_VALUE: i32 = i32::MAX;

/// Parses a `&str` to a number and makes sure it doesn't exceed a limit.
pub trait ParseNumber: Sized {
    /// Parses a number without exceeding [`MAX_PARSE_VALUE`].
    fn parse(s: &str) -> Result<Self, ParseNumberError>;

    /// Parses a number without exceeding the given limit..
    fn parse_with_limits(s: &str, limit: Self) -> Result<Self, ParseNumberError>;
}

/// All the ways that parsing with [`ParseNumber`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseNumberError {
    #[error("invalid float")]
    InvalidFloat(#[from] num::ParseFloatError),
    #[error("invalid integer")]
    InvalidInteger(#[from] num::ParseIntError),
    #[error("not a number")]
    NaN,
    #[error("value is too high")]
    NumberOverflow,
    #[error("value is too low")]
    NumberUnderflow,
}

impl ParseNumber for i32 {
    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        Self::parse_with_limits(s, MAX_PARSE_VALUE)
    }

    fn parse_with_limits(s: &str, limit: Self) -> Result<Self, ParseNumberError> {
        let n = s.parse()?;

        if n < -limit {
            Err(ParseNumberError::NumberUnderflow)
        } else if n > limit {
            Err(ParseNumberError::NumberOverflow)
        } else {
            Ok(n)
        }
    }
}

impl ParseNumber for f32 {
    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        Self::parse_with_limits(s, MAX_PARSE_VALUE as Self)
    }

    fn parse_with_limits(s: &str, limit: Self) -> Result<Self, ParseNumberError> {
        let n: Self = s.parse()?;

        if n < -limit {
            Err(ParseNumberError::NumberUnderflow)
        } else if n > limit {
            Err(ParseNumberError::NumberOverflow)
        } else if n.is_nan() {
            Err(ParseNumberError::NaN)
        } else {
            Ok(n)
        }
    }
}

impl ParseNumber for f64 {
    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        Self::parse_with_limits(s, Self::from(MAX_PARSE_VALUE))
    }

    fn parse_with_limits(s: &str, limit: Self) -> Result<Self, ParseNumberError> {
        let n: Self = s.parse()?;

        if n < -limit {
            Err(ParseNumberError::NumberUnderflow)
        } else if n > limit {
            Err(ParseNumberError::NumberOverflow)
        } else if n.is_nan() {
            Err(ParseNumberError::NaN)
        } else {
            Ok(n)
        }
    }
}
