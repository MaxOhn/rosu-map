use crate::util::Pos;

use super::{
    curve::{BorrowedCurve, Curve, CurveBuffers},
    path_type::PathType,
};

/// The path of a [`HitObjectSlider`].
///
/// [`HitObjectSlider`]: crate::section::hit_objects::HitObjectSlider
#[derive(Clone, Debug, PartialEq)]
pub struct SliderPath {
    control_points: Vec<PathControlPoint>,
    expected_dist: Option<f64>,
    curve: Option<Curve>,
}

impl SliderPath {
    /// Creates a new [`SliderPath`].
    ///
    /// The contained [`Curve`] will not necessarily be calculated yet, only
    /// when accessing it with methods such as [`SliderPath::curve`].
    pub const fn new(control_points: Vec<PathControlPoint>, expected_dist: Option<f64>) -> Self {
        Self {
            control_points,
            expected_dist,
            curve: None,
        }
    }

    /// Returns an immutable reference to the control points.
    pub fn control_points(&self) -> &[PathControlPoint] {
        &self.control_points
    }

    /// Returns the expected distance.
    pub const fn expected_dist(&self) -> Option<f64> {
        self.expected_dist
    }

    /// Returns a reference to the [`Curve`].
    ///
    /// If the curve has not yet been accessed, it needs to be calculated
    /// first.
    ///
    /// In case curves of multiple slider paths are being calculated, it is
    /// recommended to initialize [`CurveBuffers`] and pass a mutable reference
    /// of it to [`SliderPath::curve_with_bufs`] so the buffers are re-used for
    /// all sliders.
    ///
    /// Alternatively, to avoid storing the curve altogether because it will be
    /// accessed only once, using [`SliderPath::borrowed_curve`] should be
    /// preferred.
    pub fn curve(&mut self) -> &Curve {
        if let Some(ref curve) = self.curve {
            curve
        } else {
            let curve = self.calculate_curve();

            self.curve.insert(curve)
        }
    }

    /// Returns a reference to the [`Curve`].
    ///
    /// If the curve has not yet been accessed, it needs to be calculated
    /// first.
    ///
    /// In case the curve will be accessed only once, using
    /// [`SliderPath::borrowed_curve`] should be preferred.
    pub fn curve_with_bufs(&mut self, bufs: &mut CurveBuffers) -> &Curve {
        if let Some(ref curve) = self.curve {
            curve
        } else {
            let curve = self.calculate_curve_with_bufs(bufs);

            self.curve.insert(curve)
        }
    }

    /// Returns a [`BorrowedCurve`].
    ///
    /// If the curve has been calculated before, the returned curve will borrow
    /// from it. Otherwise, it will be calculated and returned **without**
    /// storing it by borrowing from the given [`CurveBuffers`].
    ///
    /// This should be preferred over [`SliderPath::curve_with_bufs`] if the
    /// curve will be accessed only once.
    pub fn borrowed_curve<'a, 'b: 'a>(&'a self, bufs: &'b mut CurveBuffers) -> BorrowedCurve<'_> {
        if let Some(ref curve) = self.curve {
            curve.as_borrowed_curve()
        } else {
            BorrowedCurve::new(&self.control_points, self.expected_dist, bufs)
        }
    }

    /// Returns a mutable reference to the control points.
    ///
    /// Note that calling this method will invalidate the stored curve
    /// so it must be recalculated on its next access.
    pub fn control_points_mut(&mut self) -> &mut Vec<PathControlPoint> {
        self.curve = None;

        &mut self.control_points
    }

    /// Returns a mutable reference to the expected distance.
    ///
    /// Note that calling this method will invalidate the stored curve
    /// so it must be recalculated on its next access.
    pub fn expected_dist_mut(&mut self) -> &mut Option<f64> {
        self.curve = None;

        &mut self.expected_dist
    }

    fn calculate_curve(&self) -> Curve {
        self.calculate_curve_with_bufs(&mut CurveBuffers::default())
    }

    fn calculate_curve_with_bufs(&self, bufs: &mut CurveBuffers) -> Curve {
        Curve::new(&self.control_points, self.expected_dist, bufs)
    }
}

/// A positional control point of a [`SliderPath`].
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PathControlPoint {
    pub pos: Pos,
    pub path_type: Option<PathType>,
}

impl PathControlPoint {
    /// Initialize a new [`PathControlPoint`].
    pub const fn new(pos: Pos) -> Self {
        Self {
            pos,
            path_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_curve() {
        let mut bufs = CurveBuffers::default();
        let mut path = SliderPath::new(Vec::new(), None);

        // freshly calculate the curve; lifetime will depend on `bufs`
        let borrowed_curve = path.borrowed_curve(&mut bufs);

        // access to let the borrow checker know it will be used
        let _ = borrowed_curve.dist();

        // calculate **and store** the curve
        let _ = path.curve_with_bufs(&mut bufs);

        // access the stored curve; lifetime will depend on `path`
        let borrowed_curve = path.borrowed_curve(&mut bufs);

        // access to let the borrow checker know it will be used
        let _ = borrowed_curve.dist();
    }
}
