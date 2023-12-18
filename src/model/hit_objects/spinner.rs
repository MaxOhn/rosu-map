use crate::util::Pos;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectSpinner {
    pub pos: Pos,
    pub duration: f64,
    pub new_combo: bool,
}
