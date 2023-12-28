/// A hold note [`HitObject`].
///
/// [`HitObject`]: crate::section::hit_objects::HitObject
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectHold {
    pub pos_x: f32,
    pub duration: f64,
}
