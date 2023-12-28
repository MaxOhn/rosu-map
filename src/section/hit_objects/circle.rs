use crate::util::Pos;

/// A circle note [`HitObject`].
///
/// [`HitObject`]: crate::section::hit_objects::HitObject
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectCircle {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
}
