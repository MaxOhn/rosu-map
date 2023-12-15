use std::num;

const MAX_PARSE_VALUE: i32 = i32::MAX;

pub trait ParseNumber: Sized {
    fn parse(s: &str) -> Result<Self, ParseNumberError>;

    fn parse_with_limits(s: &str, limit: Self) -> Result<Self, ParseNumberError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ParseNumberError {
    #[error("invalid float")]
    InvalidFloat(#[from] num::ParseFloatError),
    #[error("invalid integer")]
    InvalidInteger(#[from] num::ParseIntError),
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

impl ParseNumber for f64 {
    fn parse(s: &str) -> Result<Self, ParseNumberError> {
        Self::parse_with_limits(s, MAX_PARSE_VALUE as Self)
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
