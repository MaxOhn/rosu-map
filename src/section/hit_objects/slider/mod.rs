use crate::util::Pos;

use self::{path::SliderPath, path_type::PathType};

use super::hit_samples::HitSampleInfo;

pub mod curve;
pub mod path;
pub mod path_type;

#[derive(Clone, Debug, PartialEq)]
pub struct HitObjectSlider {
    pub pos: Pos,
    pub new_combo: bool,
    pub combo_offset: i32,
    pub path: SliderPath,
    pub node_samples: Vec<Vec<HitSampleInfo>>,
    pub repeat_count: i32,
}

impl HitObjectSlider {
    pub const fn span_count(&self) -> i32 {
        self.repeat_count + 1
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PathControlPoint {
    pub pos: Pos,
    pub path_type: Option<PathType>,
}

impl PathControlPoint {
    pub const fn new(pos: Pos) -> Self {
        Self {
            pos,
            path_type: None,
        }
    }
}
