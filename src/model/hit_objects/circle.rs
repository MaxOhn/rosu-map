use crate::util::Pos;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitObjectCircle {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
}
