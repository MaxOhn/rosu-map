use std::{borrow::Cow, cmp::Ordering, convert::identity, f64::consts::PI, iter, mem};

use crate::util::Pos;

use super::{
    path_type::{PathType, SplineType},
    PathControlPoint,
};

const BEZIER_TOLERANCE: f32 = 0.25;
const CATMULL_DETAIL: usize = 50;
const CIRCULAR_ARC_TOLERANCE: f32 = 0.1;

/// Collection of buffers required to calculate a [`Curve`].
#[derive(Clone, Debug, Default)]
pub struct CurveBuffers {
    path: Vec<Pos>,
    lengths: Vec<f64>,
    vertices: Vec<Pos>,
    bezier: BezierBuffers,
}

#[derive(Clone, Debug, Default)]
struct BezierBuffers {
    left: Vec<Pos>,
    right: Vec<Pos>,
    midpoints: Vec<Pos>,
    left_child: Vec<Pos>,
}

impl BezierBuffers {
    /// Fill the buffers with new elements until a
    /// length of `len` is reached. Does nothing if `len`
    /// is already smaller than the current buffer size.
    fn extend_exact(&mut self, len: usize) {
        if len <= self.left.len() {
            return;
        }

        let additional = len - self.left.len();

        self.left
            .extend(iter::repeat(Pos::default()).take(additional));
        self.right
            .extend(iter::repeat(Pos::default()).take(additional));
        self.midpoints
            .extend(iter::repeat(Pos::default()).take(additional));
        self.left_child
            .extend(iter::repeat(Pos::default()).take(additional));
    }
}

struct CircularArcProperties {
    theta_start: f64,
    theta_range: f64,
    direction: f64,
    radius: f32,
    centre: Pos,
}

/// A curve with owned lists of path points and segment lengths.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    path: Vec<Pos>,
    lengths: Vec<f64>,
}

impl Curve {
    /// Create a new [`Curve`].
    pub fn new(
        points: &[PathControlPoint],
        expected_len: Option<f64>,
        bufs: &mut CurveBuffers,
    ) -> Self {
        calculate_path(points, bufs);
        calculate_length(points, bufs, expected_len);

        Self {
            path: mem::take(&mut bufs.path),
            lengths: mem::take(&mut bufs.lengths),
        }
    }

    /// Path points of the [`Curve`].
    pub fn path(&self) -> &[Pos] {
        &self.path
    }

    /// The [`Curve`]'s length at each path point.
    pub fn lengths(&self) -> &[f64] {
        &self.lengths
    }

    /// The interpolated position at the given progress.
    pub fn position_at(&self, progress: f64) -> Pos {
        position_at(&self.path, &self.lengths, progress)
    }

    /// Value between 0.0 and the curve's distance, depending on the given
    /// progress between 0.0 and 1.0.
    pub fn progress_to_dist(&self, progress: f64) -> f64 {
        progress_to_dist(&self.lengths, progress)
    }

    /// The total distance of the [`Curve`].
    pub fn dist(&self) -> f64 {
        dist(&self.lengths)
    }

    /// The index into [`Curve::lengths`] to reach the distance `d`.
    pub fn idx_of_dist(&self, d: f64) -> usize {
        idx_of_dist(&self.lengths, d)
    }

    /// Interpolate the position between the i'th and i+1'th path position
    /// at distance `d`.
    pub fn interpolate_vertices(&self, i: usize, d: f64) -> Pos {
        interpolate_vertices(&self.path, &self.lengths, i, d)
    }

    /// Cheaply transform [`Curve`] into [`BorrowedCurve`].
    pub fn as_borrowed_curve(&self) -> BorrowedCurve<'_> {
        BorrowedCurve {
            path: &self.path,
            lengths: &self.lengths,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A curve with borrowed lists of path points and segment lengths.
pub struct BorrowedCurve<'bufs> {
    path: &'bufs [Pos],
    lengths: &'bufs [f64],
}

impl<'bufs> BorrowedCurve<'bufs> {
    /// Create a new [`BorrowedCurve`] which borrows from [`CurveBuffers`].
    pub fn new(
        points: &[PathControlPoint],
        expected_len: Option<f64>,
        bufs: &'bufs mut CurveBuffers,
    ) -> Self {
        calculate_path(points, bufs);
        calculate_length(points, bufs, expected_len);

        Self {
            path: &bufs.path,
            lengths: &bufs.lengths,
        }
    }

    /// Path points of the [`BorrowedCurve`].
    pub const fn path(&self) -> &[Pos] {
        self.path
    }

    /// The [`BorrowedCurve`]'s length at each path point.
    pub const fn lengths(&self) -> &[f64] {
        self.lengths
    }

    /// The interpolated position at the given progress.
    pub fn position_at(&self, progress: f64) -> Pos {
        position_at(self.path, self.lengths, progress)
    }

    /// Value between 0.0 and the curve's distance, depending on the given
    /// progress between 0.0 and 1.0.
    pub fn progress_to_dist(&self, progress: f64) -> f64 {
        progress_to_dist(self.lengths, progress)
    }

    /// The total distance of the [`BorrowedCurve`].
    pub fn dist(&self) -> f64 {
        dist(self.lengths)
    }

    /// The index into [`BorrowedCurve::lengths`] to reach the distance `d`.
    pub fn idx_of_dist(&self, d: f64) -> usize {
        idx_of_dist(self.lengths, d)
    }

    /// Interpolate the position between the i'th and i+1'th path position
    /// at distance `d`.
    pub fn interpolate_vertices(&self, i: usize, d: f64) -> Pos {
        interpolate_vertices(self.path, self.lengths, i, d)
    }

    /// Allocates the borrowed lists to transform [`BorrowedCurve`] into [`Curve`].
    pub fn to_owned_curve(&self) -> Curve {
        Curve {
            path: self.path.to_owned(),
            lengths: self.lengths.to_owned(),
        }
    }
}

fn position_at(path: &[Pos], lengths: &[f64], progress: f64) -> Pos {
    let d = progress_to_dist(lengths, progress);
    let i = idx_of_dist(lengths, d);

    interpolate_vertices(path, lengths, i, d)
}

fn progress_to_dist(lengths: &[f64], progress: f64) -> f64 {
    progress.clamp(0.0, 1.0) * dist(lengths)
}

fn dist(lengths: &[f64]) -> f64 {
    lengths.last().copied().unwrap_or(0.0)
}

fn idx_of_dist(lengths: &[f64], d: f64) -> usize {
    lengths
        .binary_search_by(|len| len.partial_cmp(&d).unwrap_or(Ordering::Equal))
        .map_or_else(identity, identity)
}

fn interpolate_vertices(path: &[Pos], lengths: &[f64], i: usize, d: f64) -> Pos {
    if path.is_empty() {
        return Pos::default();
    }

    let p1 = if i == 0 {
        return path[0];
    } else if let Some(p) = path.get(i) {
        *p
    } else {
        return path[path.len() - 1];
    };

    let p0 = path[i - 1];

    let d0 = lengths[i - 1];
    let d1 = lengths[i];

    // * Avoid division by an almost-zero number in case
    // * two points are extremely close to each other
    if (d0 - d1).abs() <= f64::EPSILON {
        return p0;
    }

    let w = (d - d0) / (d1 - d0);

    p0 + (p1 - p0) * w as f32
}

fn calculate_path(points: &[PathControlPoint], bufs: &mut CurveBuffers) {
    bufs.path.clear();

    if points.is_empty() {
        return;
    }

    let CurveBuffers {
        vertices,
        bezier,
        path,
        ..
    } = bufs;

    vertices.clear();
    vertices.extend(points.iter().map(|p| p.pos));

    let mut start = 0;

    for i in 0..points.len() {
        if points[i].path_type.is_none() && i < points.len() - 1 {
            continue;
        }

        // * The current vertex ends the segment
        let segment_vertices = &vertices[start..=i];
        let segment_kind = points[start].path_type.unwrap_or(PathType::LINEAR);

        calculate_subpath(path, segment_vertices, segment_kind, bezier);

        // * Start the new segment at the current vertex
        start = i;
    }

    path.dedup();
}

fn calculate_length(
    points: &[PathControlPoint],
    bufs: &mut CurveBuffers,
    expected_len: Option<f64>,
) {
    let CurveBuffers {
        path,
        lengths: cumulative_len,
        ..
    } = bufs;

    cumulative_len.clear();
    let mut calculated_len = 0.0;
    cumulative_len.reserve(path.len());
    cumulative_len.push(0.0);

    let length_iter = path.iter().zip(path.iter().skip(1)).map(|(&curr, &next)| {
        calculated_len += f64::from((next - curr).length());

        calculated_len
    });

    cumulative_len.extend(length_iter);

    if let Some(expected_len) =
        expected_len.filter(|&len| (calculated_len - len).abs() >= f64::EPSILON)
    {
        // * In osu-stable, if the last two control points of a slider are equal, extension is not performed
        let condition_opt = points
            .len()
            .checked_sub(2)
            .and_then(|i| points.get(i..))
            .filter(|suffix| suffix[0].pos == suffix[1].pos && expected_len > calculated_len);

        if condition_opt.is_some() {
            cumulative_len.push(calculated_len);

            return;
        }

        // Shortcut when it's just (0,0) since there's nothing to do anyway
        if cumulative_len.len() == 1 {
            return;
        }

        // * The last length is always incorrect
        cumulative_len.pop();

        let last_valid = cumulative_len
            .iter()
            .rev()
            .position(|l| *l < expected_len)
            .map_or(0, |idx| cumulative_len.len() - idx);

        // * The path will be shortened further, in which case we should trim
        // * any more unnecessary lengths and their associated path segments
        if last_valid < cumulative_len.len() {
            cumulative_len.truncate(last_valid);
            path.truncate(last_valid + 1);

            if cumulative_len.is_empty() {
                // * The expected distance is negative or zero
                // * Perhaps negative path lengths should be disallowed altogether
                cumulative_len.push(0.0);

                return;
            }
        }

        let end_idx = cumulative_len.len();
        let prev_idx = end_idx - 1;

        // * The direction of the segment to shorten or lengthen
        let dir = (path[end_idx] - path[prev_idx]).normalize();

        path[end_idx] = path[prev_idx] + dir * (expected_len - cumulative_len[prev_idx]) as f32;
        cumulative_len.push(expected_len);
    }
}

fn calculate_subpath(
    path: &mut Vec<Pos>,
    sub_points: &[Pos],
    path_type: PathType,
    bufs: &mut BezierBuffers,
) {
    match path_type.kind {
        SplineType::Catmull => approximate_catmull(path, sub_points),
        SplineType::Linear => approximate_linear(path, sub_points),
        SplineType::PerfectCurve => {
            if let [a, b, c] = sub_points {
                if approximate_circular_arc(path, *a, *b, *c) {
                    return;
                }
            }

            approximate_bezier(path, sub_points, bufs);
        }
        SplineType::BSpline => approximate_bezier(path, sub_points, bufs),
    }
}

fn approximate_bezier(path: &mut Vec<Pos>, points: &[Pos], bufs: &mut BezierBuffers) {
    bufs.extend_exact(points.len());

    approximate_bspline(path, points, bufs);
}

fn approximate_catmull(path: &mut Vec<Pos>, points: &[Pos]) {
    if points.len() == 1 {
        return;
    }

    path.reserve_exact((points.len() - 1) * CATMULL_DETAIL * 2);

    // Handle first iteration distinctly because of v1
    let v1 = points[0];
    let v2 = points[0];
    let v3 = points.get(1).copied().unwrap_or(v2);
    let v4 = points.get(2).copied().unwrap_or_else(|| v3 * 2.0 - v2);

    catmull_subpath(path, v1, v2, v3, v4);

    // Remaining iterations
    for (i, (&v1, &v2)) in (2..points.len()).zip(points.iter().zip(points.iter().skip(1))) {
        let v3 = points.get(i).copied().unwrap_or_else(|| v2 * 2.0 - v1);
        let v4 = points.get(i + 1).copied().unwrap_or_else(|| v3 * 2.0 - v2);

        catmull_subpath(path, v1, v2, v3, v4);
    }
}

fn approximate_linear(path: &mut Vec<Pos>, points: &[Pos]) {
    path.extend(points);
}

fn approximate_circular_arc(path: &mut Vec<Pos>, a: Pos, b: Pos, c: Pos) -> bool {
    let Some(pr) = circular_arc_properties(a, b, c) else {
        return false;
    };

    // * We select the amount of points for the approximation by requiring the discrete curvature
    // * to be smaller than the provided tolerance. The exact angle required to meet the tolerance
    // * is: 2 * Math.Acos(1 - TOLERANCE / r)
    // * The special case is required for extremely short sliders where the radius is smaller than
    // * the tolerance. This is a pathological rather than a realistic case.
    let amount_points = if 2.0 * pr.radius <= CIRCULAR_ARC_TOLERANCE {
        2
    } else {
        let divisor = 2.0 * (1.0 - CIRCULAR_ARC_TOLERANCE / pr.radius).acos();

        // In C# it holds `(int)Infinity == -2147483648` whereas in Rust it's 2147483647
        // so we need to workaround this edge case, see map id 2568364
        if divisor.abs() <= f32::EPSILON {
            2
        } else {
            ((pr.theta_range / f64::from(divisor)).ceil() as usize).max(2)
        }
    };

    path.reserve_exact(amount_points);
    let divisor = (amount_points - 1) as f64;
    let directed_range = pr.direction * pr.theta_range;

    let subpath = (0..amount_points).map(|i| {
        let fract = i as f64 / divisor;
        let theta = pr.theta_start + fract * directed_range;
        let (sin, cos) = theta.sin_cos();

        let origin = Pos {
            x: cos as f32,
            y: sin as f32,
        };

        pr.centre + origin * pr.radius
    });

    path.extend(subpath);

    true
}

fn approximate_bspline(path: &mut Vec<Pos>, points: &[Pos], bufs: &mut BezierBuffers) {
    let p = points.len();

    let mut to_flatten = Vec::new();
    let mut free_bufs = Vec::new();

    // In osu!lazer's code, `p` is always 0 so the first big `if` can be omitted

    to_flatten.push(Cow::Borrowed(points));

    // * "toFlatten" contains all the curves which are not yet approximated well enough.
    // * We use a stack to emulate recursion without the risk of running into a stack overflow.
    // * (More specifically, we iteratively and adaptively refine our curve with a
    // * <a href="https://en.wikipedia.org/wiki/Depth-first_search">Depth-first search</a>
    // * over the tree resulting from the subdivisions we make.)

    let BezierBuffers {
        left,
        right,
        midpoints,
        left_child,
    } = bufs;

    while let Some(mut parent) = to_flatten.pop() {
        if bezier_is_flat_enough(&parent) {
            // * If the control points we currently operate on are sufficiently "flat", we use
            // * an extension to De Casteljau's algorithm to obtain a piecewise-linear approximation
            // * of the bezier curve represented by our control points, consisting of the same amount
            // * of points as there are control points.
            bezier_approximate(&parent, path, left, right, midpoints);
            free_bufs.push(parent);

            continue;
        }

        // * If we do not yet have a sufficiently "flat" (in other words, detailed) approximation we keep
        // * subdividing the curve we are currently operating on.
        let mut right_child = free_bufs
            .pop()
            .unwrap_or_else(|| Cow::Owned(vec![Pos::default(); p]));

        bezier_subdivide(&parent, left_child, right_child.to_mut(), midpoints);

        // * We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
        parent.to_mut().copy_from_slice(&left_child[..p]);

        to_flatten.push(right_child);
        to_flatten.push(parent);
    }

    path.push(points[p - 1]);
}

fn bezier_is_flat_enough(points: &[Pos]) -> bool {
    let limit = BEZIER_TOLERANCE * BEZIER_TOLERANCE * 4.0;

    !points
        .iter()
        .zip(points.iter().skip(1))
        .zip(points.iter().skip(2))
        .any(|((&prev, &curr), &next)| (prev - curr * 2.0 + next).length_squared() > limit)
}

fn bezier_subdivide(points: &[Pos], l: &mut [Pos], r: &mut [Pos], midpoints: &mut [Pos]) {
    let count = points.len();
    midpoints[..count].copy_from_slice(&points[..count]);

    for i in (1..count).rev() {
        l[count - i - 1] = midpoints[0];
        r[i] = midpoints[i];

        for j in 0..i {
            midpoints[j] = (midpoints[j] + midpoints[j + 1]) / 2.0;
        }
    }

    l[count - 1] = midpoints[0];
    r[0] = midpoints[0];
}

// * https://en.wikipedia.org/wiki/De_Casteljau%27s_algorithm
fn bezier_approximate(
    points: &[Pos],
    path: &mut Vec<Pos>,
    l: &mut [Pos],
    r: &mut [Pos],
    midpoints: &mut [Pos],
) {
    let count = points.len();

    bezier_subdivide(points, l, r, midpoints);
    path.push(points[0]);

    let l = &l[..count];
    let r = &r[1..count];

    let subpath = l
        .iter()
        .chain(r)
        .skip(1)
        .zip(l.iter().chain(r).skip(2))
        .zip(l.iter().chain(r).skip(3))
        .step_by(2)
        .map(|((&prev, &curr), &next)| (prev + curr * 2.0 + next) * 0.25);

    path.extend(subpath);
}

fn catmull_subpath(path: &mut Vec<Pos>, v1: Pos, v2: Pos, v3: Pos, v4: Pos) {
    let x1 = 2.0 * v2.x;
    let x2 = -v1.x + v3.x;
    let x3 = 2.0 * v1.x - 5.0 * v2.x + 4.0 * v3.x - v4.x;
    let x4 = -v1.x + 3.0 * (v2.x - v3.x) + v4.x;

    let y1 = 2.0 * v2.y;
    let y2 = -v1.y + v3.y;
    let y3 = 2.0 * v1.y - 5.0 * v2.y + 4.0 * v3.y - v4.y;
    let y4 = -v1.y + 3.0 * (v2.y - v3.y) + v4.y;

    let catmull_detail = CATMULL_DETAIL as f32;

    let subpath = (0..CATMULL_DETAIL).flat_map(|c| {
        let c = c as f32;
        let t1 = c / catmull_detail;
        let t2 = t1 * t1;
        let t3 = t2 * t1;

        let pos1 = Pos {
            x: 0.5 * (x1 + x2 * t1 + x3 * t2 + x4 * t3),
            y: 0.5 * (y1 + y2 * t1 + y3 * t2 + y4 * t3),
        };

        let t1 = (c + 1.0) / catmull_detail;
        let t2 = t1 * t1;
        let t3 = t2 * t1;

        let pos2 = Pos {
            x: 0.5 * (x1 + x2 * t1 + x3 * t2 + x4 * t3),
            y: 0.5 * (y1 + y2 * t1 + y3 * t2 + y4 * t3),
        };

        iter::once(pos1).chain(iter::once(pos2))
    });

    path.extend(subpath);
}

fn circular_arc_properties(a: Pos, b: Pos, c: Pos) -> Option<CircularArcProperties> {
    // * If we have a degenerate triangle where a side-length is almost zero,
    // * then give up and fallback to a more numerically stable method.
    if ((b.y - a.y) * (c.x - a.x) - (b.x - a.x) * (c.y - a.y)).abs() <= f32::EPSILON {
        return None;
    }

    // * See: https://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates_2
    let d = 2.0 * (a.x * (b - c).y + b.x * (c - a).y + c.x * (a - b).y);
    let a_sq = a.length_squared();
    let b_sq = b.length_squared();
    let c_sq = c.length_squared();

    let centre = Pos {
        x: (a_sq * (b - c).y + b_sq * (c - a).y + c_sq * (a - b).y) / d,
        y: (a_sq * (c - b).x + b_sq * (a - c).x + c_sq * (b - a).x) / d,
    };

    let d_a = a - centre;
    let d_c = c - centre;

    let radius = d_a.length();

    let theta_start = f64::from(d_a.y).atan2(f64::from(d_a.x));
    let mut theta_end = f64::from(d_c.y).atan2(f64::from(d_c.x));

    while theta_end < theta_start {
        theta_end += 2.0 * PI;
    }

    let mut direction = 1.0;
    let mut theta_range = theta_end - theta_start;

    // * Decide in which direction to draw the circle,
    // * depending on which side of AC B lies.
    let mut ortho_a_to_c = c - a;

    ortho_a_to_c = Pos {
        x: ortho_a_to_c.y,
        y: -ortho_a_to_c.x,
    };

    if ortho_a_to_c.dot(b - a) < 0.0 {
        direction = -direction;
        theta_range = 2.0 * PI - theta_range;
    }

    Some(CircularArcProperties {
        theta_start,
        theta_range,
        direction,
        radius,
        centre,
    })
}
