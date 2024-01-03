use std::num::NonZeroI32;

/// The type of a [`SliderPath`]'s segment.
///
/// [`SliderPath`]: crate::section::hit_objects::SliderPath
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PathType {
    pub kind: SplineType,
    pub degree: Option<NonZeroI32>,
}

impl PathType {
    pub const CATMULL: Self = Self::new(SplineType::Catmull);
    pub const BEZIER: Self = Self::new(SplineType::BSpline);
    pub const LINEAR: Self = Self::new(SplineType::Linear);
    pub const PERFECT_CURVE: Self = Self::new(SplineType::PerfectCurve);

    /// Initialize a new [`PathType`] without a degree.
    pub const fn new(kind: SplineType) -> Self {
        Self { kind, degree: None }
    }

    /// Initialize a new `BSpline` [`PathType`].
    pub const fn new_b_spline(degree: NonZeroI32) -> Self {
        Self {
            kind: SplineType::BSpline,
            degree: Some(degree),
        }
    }

    /// Parse a string into a [`PathType`].
    ///
    /// The string should be of the form `"B<optional integer>" | "L" | "P"`
    /// (without the `<>`). Otherwise, a catmull path type is returned.
    pub fn new_from_str(input: &str) -> Self {
        let mut chars = input.chars();

        match chars.next() {
            Some('B') => {
                if let Ok(Some(degree)) = chars.as_str().parse().map(NonZeroI32::new) {
                    if degree.is_positive() {
                        return Self::new_b_spline(degree);
                    }
                }

                Self::BEZIER
            }
            Some('L') => Self::LINEAR,
            Some('P') => Self::PERFECT_CURVE,
            _ => Self::CATMULL,
        }
    }
}

/// The specific type of a [`PathType`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SplineType {
    #[default]
    Catmull,
    BSpline,
    Linear,
    PerfectCurve,
}
