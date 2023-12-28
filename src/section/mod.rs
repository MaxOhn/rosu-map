/// Decode-related types for the `[General]` section.
pub mod general;

/// Decode-related types for the `[Editor]` section.
pub mod editor;

/// Decode-related types for the `[Difficulty]` section.
pub mod difficulty;

/// Decode-related types for the `[Metadata]` section.
pub mod metadata;

/// Decode-related types for the `[Events]` section.
pub mod events;

/// Decode-related types for the `[TimingPoints]` section.
pub mod timing_points;

/// Decode-related types for the `[Colours]` section.
pub mod colors;

/// Decode-related types for the `[HitObjects]` section.
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
}

impl Section {
    /// Try to parse a [`Section`].
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

/// The error of a failed parsing of a section key.
#[derive(Debug, thiserror::Error)]
#[error("unknown key")]
pub struct UnknownKeyError;

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
