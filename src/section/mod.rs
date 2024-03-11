/// Types for the `[General]` section.
pub mod general;

/// Types for the `[Editor]` section.
pub mod editor;

/// Types for the `[Difficulty]` section.
pub mod difficulty;

/// Types for the `[Metadata]` section.
pub mod metadata;

/// Types for the `[Events]` section.
pub mod events;

/// Types for the `[TimingPoints]` section.
pub mod timing_points;

/// Types for the `[Colours]` section.
pub mod colors;

/// Types for the `[HitObjects]` section.
pub mod hit_objects;

/// All sections in a `.osu` file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    General,
    Editor,
    Metadata,
    Difficulty,
    Events,
    TimingPoints,
    Colors,
    HitObjects,
    Variables,
    CatchTheBeat,
    Mania,
}

impl Section {
    /// Try to parse a [`Section`].
    pub fn try_from_line(line: &str) -> Option<Self> {
        let section = line.strip_prefix('[')?.strip_suffix(']')?;

        let section = match section {
            "General" => Self::General,
            "Editor" => Self::Editor,
            "Metadata" => Self::Metadata,
            "Difficulty" => Self::Difficulty,
            "Events" => Self::Events,
            "TimingPoints" => Self::TimingPoints,
            "Colours" => Self::Colors,
            "HitObjects" => Self::HitObjects,
            "Variables" => Self::Variables,
            "CatchTheBeat" => Self::CatchTheBeat,
            "Mania" => Self::Mania,
            _ => return None,
        };

        Some(section)
    }
}

thiserror! {
    #[error("unknown key")]
    /// The error when failing to parse a section key.
    #[derive(Debug)]
    pub struct UnknownKeyError;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_valid_sections() {
        assert_eq!(Section::try_from_line("[General]"), Some(Section::General));
        assert_eq!(
            Section::try_from_line("[Difficulty]"),
            Some(Section::Difficulty)
        );
        assert_eq!(
            Section::try_from_line("[HitObjects]"),
            Some(Section::HitObjects)
        );
    }

    #[test]
    fn requires_brackets() {
        assert_eq!(Section::try_from_line("General"), None);
        assert_eq!(Section::try_from_line("[General"), None);
        assert_eq!(Section::try_from_line("General]"), None);
    }

    #[test]
    fn denies_invalid_sections() {
        assert_eq!(Section::try_from_line("abc"), None);
        assert_eq!(Section::try_from_line("HitObject"), None);
    }
}
