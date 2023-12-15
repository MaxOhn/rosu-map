pub mod colors;
pub mod difficulty;
pub mod editor;
pub mod events;
pub mod general;
pub mod hit_objects;
pub mod metadata;
pub mod timing_points;

#[derive(Copy, Clone)]
pub enum Section {
    General,
    Editor,
    Metadata,
    Difficulty,
    Events,
    TimingPoints,
    Colors,
    HitObjects,
}

impl Section {
    pub fn try_from_line(line: &str) -> Option<Self> {
        let section = line.strip_prefix('[')?.strip_suffix(']')?;

        let section = match section {
            "General" => Section::General,
            "Editor" => Section::Editor,
            "Metadata" => Section::Metadata,
            "Difficulty" => Section::Difficulty,
            "Events" => Section::Events,
            "TimingPoints" => Section::TimingPoints,
            "Colours" => Section::Colors,
            "HitObjects" => Section::HitObjects,
            _ => return None,
        };

        Some(section)
    }
}
