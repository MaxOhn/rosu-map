use std::{io::Cursor, path::Path, str::FromStr};

use crate::{
    error::{ParseError, ParseResult},
    reader::Reader,
};

#[derive(Debug, Default)]
pub struct Beatmap {
    pub format_version: i32,

    // General

    // Editor
    pub bookmarks: Vec<i32>,
    pub distance_spacing: f32,
    pub beat_divisor: i32,
    pub grid_size: i32,
    pub timeline_zoom: f32,

    // Metadata
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: Vec<String>,
    pub beatmap_id: u32,
    pub beatmap_set_id: i32,

    // Difficulty
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
    // Events

    // TimingPoints

    // Colours

    // HitObjects
}

impl Beatmap {
    pub fn from_path(path: impl AsRef<Path>) -> ParseResult<Self> {
        std::fs::read_to_string(path)?.parse()
    }

    pub fn from_bytes(bytes: &[u8]) -> ParseResult<Self> {
        parse::parse_input(Reader::new(Cursor::new(bytes)))
    }
}

impl FromStr for Beatmap {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse_input(Reader::new(Cursor::new(s)))
    }
}

mod parse {
    #![allow(unused)]

    use std::{io::Read, ops::ControlFlow, path::Path, str::Lines};

    use crate::{
        beatmap::Beatmap,
        error::{ParseError, ParseResult},
        reader::Reader,
        section::Section,
    };

    pub fn parse_path(path: impl AsRef<Path>) -> ParseResult<Beatmap> {
        todo!()
    }

    pub fn parse_input<R: Read>(mut reader: Reader<R>) -> ParseResult<Beatmap> {
        // TODO: UTF-16

        let mut buf = String::with_capacity(32);

        let mut map = Beatmap {
            format_version: parse_version(&mut reader, &mut buf)?,
            ..Default::default()
        };

        parse_sections(&mut reader, &mut map, &mut buf)?;

        Ok(map)
    }

    fn parse_version<R: Read>(reader: &mut Reader<R>, buf: &mut String) -> ParseResult<i32> {
        const LATEST_VERSION: i32 = 14;
        const VERSION_PREFIX: &str = "osu file format v";

        match reader
            .read_line(buf)?
            .and_then(|line| line.strip_prefix(VERSION_PREFIX))
        {
            Some(suffix) => suffix.parse().map_err(|_| ParseError::InvalidInteger),
            None => Ok(LATEST_VERSION),
        }
    }

    type Flow = ControlFlow<(), Section>;

    fn parse_sections<R: Read>(
        reader: &mut Reader<R>,
        map: &mut Beatmap,
        buf: &mut String,
    ) -> ParseResult<()> {
        let mut section = loop {
            match reader.read_line(buf)?.map(Section::try_from_line) {
                Some(Some(section)) => break section,
                Some(None) => {}
                None => return Ok(()),
            }
        };

        loop {
            let flow = match section {
                Section::General => parse_section(reader, map, buf, parse_general)?,
                Section::Editor => parse_section(reader, map, buf, parse_editor)?,
                Section::Metadata => parse_metadata(reader, map, buf)?,
                Section::Difficulty => parse_section(reader, map, buf, parse_difficulty)?,
                Section::Events => parse_section(reader, map, buf, parse_events)?,
                Section::TimingPoints => parse_section(reader, map, buf, parse_timing_points)?,
                Section::Colours => parse_section(reader, map, buf, parse_colours)?,
                Section::HitObjects => parse_section(reader, map, buf, parse_hit_objects)?,
            };

            match flow {
                Flow::Continue(next) => section = next,
                Flow::Break(_) => break,
            }
        }

        Ok(())
    }

    fn parse_section<R, F>(
        reader: &mut Reader<R>,
        map: &mut Beatmap,
        buf: &mut String,
        f: F,
    ) -> ParseResult<Flow>
    where
        R: Read,
        F: Fn(&mut Beatmap, &str) -> ParseResult<()>,
    {
        while let Some(line) = reader.read_line(buf)? {
            if let Some(next) = Section::try_from_line(line) {
                return Ok(Flow::Continue(next));
            }

            f(map, line)?;
        }

        Ok(Flow::Break(()))
    }

    fn parse_general(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        let (key, value) = line.split_once(':').ok_or_else(|| todo!())?;

        match key.trim() {
            _ => {}
        }

        Ok(())
    }

    fn parse_editor(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        let (key, value) = line.split_once(':').ok_or_else(|| todo!())?;

        match key.trim() {
            "Bookmarks" => {
                map.bookmarks = value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .map_err(|_| todo!())?
            }
            "DistanceSpacing" => {
                map.distance_spacing = value.trim().parse().map_err(|_| todo!())?
            }
            "BeatDivisor" => map.beat_divisor = value.trim().parse().map_err(|_| todo!())?,
            "GridSize" => map.grid_size = value.trim().parse().map_err(|_| todo!())?,
            "TimelineZoom" => map.timeline_zoom = value.trim().parse().map_err(|_| todo!())?,
            _ => todo!(),
        }

        Ok(())
    }

    // Distinct from the others because comments aren't stripped
    // since song metadata may contain "//" as valid data
    fn parse_metadata<R>(
        reader: &mut Reader<R>,
        map: &mut Beatmap,
        buf: &mut String,
    ) -> ParseResult<Flow>
    where
        R: Read,
    {
        while let Some(line) = reader.read_line_with_comments(buf)? {
            if let Some(next) = Section::try_from_line(line) {
                return Ok(Flow::Continue(next));
            }

            let (key, value) = line.split_once(':').ok_or_else(|| todo!())?;

            match key.trim() {
                "Title" => map.title = value.trim().to_owned(),
                "TitleUnicode" => map.title_unicode = value.trim().to_owned(),
                "Artist" => map.artist = value.trim().to_owned(),
                "ArtistUnicode" => map.artist_unicode = value.trim().to_owned(),
                "Creator" => map.creator = value.trim().to_owned(),
                "Version" => map.version = value.trim().to_owned(),
                "Source" => map.source = value.trim().to_owned(),
                "Tags" => {
                    map.tags = value
                        .split(' ')
                        .filter(|tag| !tag.is_empty())
                        .map(str::to_owned)
                        .collect()
                }
                "BeatmapID" => map.beatmap_id = value.trim().parse().map_err(|_| todo!())?,
                "BeatmapSetID" => map.beatmap_set_id = value.trim().parse().map_err(|_| todo!())?,
                _ => todo!(),
            }
        }

        Ok(Flow::Break(()))
    }

    fn parse_difficulty(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        let (key, value) = line.split_once(':').ok_or_else(|| todo!())?;

        match key.trim() {
            "HPDrainRate" => map.hp_drain_rate = value.trim().parse().map_err(|_| todo!())?,
            "CircleSize" => map.circle_size = value.trim().parse().map_err(|_| todo!())?,
            "OverallDifficulty" => {
                map.overall_difficulty = value.trim().parse().map_err(|_| todo!())?
            }
            "ApproachRate" => map.approach_rate = value.trim().parse().map_err(|_| todo!())?,
            "SliderMultiplier" => {
                map.slider_multiplier = value.trim().parse().map_err(|_| todo!())?
            }
            "SliderTickRate" => map.slider_tick_rate = value.trim().parse().map_err(|_| todo!())?,
            _ => todo!(),
        }

        Ok(())
    }

    fn parse_events(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        Ok(())
    }

    fn parse_timing_points(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        Ok(())
    }

    fn parse_colours(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        Ok(())
    }

    fn parse_hit_objects(map: &mut Beatmap, line: &str) -> ParseResult<()> {
        Ok(())
    }
}
