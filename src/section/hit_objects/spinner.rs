use crate::util::Pos;

/// A spinner [`HitObject`].
///
/// [`HitObject`]: crate::section::hit_objects::HitObject
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectSpinner {
    pub pos: Pos,
    pub duration: f64,
    pub new_combo: bool,
}
