use std::num::NonZeroI32;

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

    const fn new(kind: SplineType) -> Self {
        Self { kind, degree: None }
    }

    const fn new_b_spline(degree: NonZeroI32) -> Self {
        Self {
            kind: SplineType::BSpline,
            degree: Some(degree),
        }
    }

    pub fn new_from_str(input: &str) -> Self {
        match input.chars().next() {
            Some('B') => {
                if let Ok(Some(degree)) = input[1..].parse().map(NonZeroI32::new) {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SplineType {
    #[default]
    Catmull,
    BSpline,
    Linear,
    PerfectCurve,
}
