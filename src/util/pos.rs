use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Simple `(x, y)` coordinate / vector
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pos {
    /// Position on the x-axis.
    pub x: f32,
    /// Position on the y-axis.
    pub y: f32,
}

impl Pos {
    /// Create a new position.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Return the position's length squared.
    pub fn length_squared(&self) -> f32 {
        self.dot(*self)
    }

    /// Return the position's length.
    pub fn length(&self) -> f32 {
        f64::from(self.x * self.x + self.y * self.y).sqrt() as f32
    }

    /// Return the dot product.
    pub fn dot(&self, other: Self) -> f32 {
        (self.x * other.x) + (self.y * other.y)
    }

    /// Return the distance to another position.
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    /// Normalize the coordinates with respect to the vector's length.
    #[must_use]
    pub fn normalize(mut self) -> Self {
        let scale = self.length().recip();
        self.x *= scale;
        self.y *= scale;

        self
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Pos {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Pos {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Pos {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Pos {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {})", self.x, self.y)
    }
}
