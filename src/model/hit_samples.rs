use std::{
    cmp,
    fmt::{Display, Formatter, Result as FmtResult},
    num::{NonZeroU32, ParseIntError},
    ops::{BitAnd, BitAndAssign},
    str::{FromStr, Split},
};

use crate::util::{ParseNumber, ParseNumberError, StrExt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitSampleInfo {
    pub name: Option<HitSampleInfoName>,
    pub filename: Option<String>,
    pub bank: SampleBank,
    pub suffix: Option<NonZeroU32>,
    pub volume: i32,
    pub custom_sample_bank: i32,
    pub bank_specified: bool,
    pub is_layered: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HitSampleInfoName {
    Normal,
    Whistle,
    Finish,
    Clap,
}

impl HitSampleInfoName {
    pub const fn to_lowercase_str(self) -> &'static str {
        match self {
            HitSampleInfoName::Normal => "hitnormal",
            HitSampleInfoName::Whistle => "hitwhistle",
            HitSampleInfoName::Finish => "hitfinish",
            HitSampleInfoName::Clap => "hitclap",
        }
    }
}

impl Display for HitSampleInfoName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.to_lowercase_str())
    }
}

impl HitSampleInfo {
    pub const BANK_NORMAL: SampleBank = SampleBank::Normal;
    pub const BANK_SOFT: SampleBank = SampleBank::Soft;
    pub const BANK_DRUM: SampleBank = SampleBank::Drum;

    pub fn new(
        name: Option<HitSampleInfoName>,
        bank: Option<SampleBank>,
        custom_sample_bank: i32,
        volume: i32,
    ) -> Self {
        Self {
            name,
            filename: None,
            bank: bank.unwrap_or(Self::BANK_NORMAL),
            suffix: (custom_sample_bank >= 2)
                .then(|| 
                    // SAFETY: The value is guaranteed to be >= 2
                    unsafe { NonZeroU32::new_unchecked(custom_sample_bank as u32) }
                ),
            volume,
            custom_sample_bank,
            bank_specified: bank.is_some(),
            is_layered: false,
        }
    }

    pub fn lookup_name(&self) -> Option<String> {
        self.name.map(|name| {
            if let Some(ref suffix) = self.suffix {
                format!("Gameplay/{}-{name}{suffix}", self.bank)
            } else {
                format!("Gameplay/{}-{name}", self.bank)
            }
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SampleBank {
    #[default]
    None,
    Normal,
    Soft,
    Drum,
}

impl Display for SampleBank {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.to_lowercase_str())
    }
}

impl SampleBank {
    pub const fn to_lowercase_str(self) -> &'static str {
        match self {
            SampleBank::None => "none",
            SampleBank::Normal => "normal",
            SampleBank::Soft => "soft",
            SampleBank::Drum => "drum",
        }
    }

    pub fn from_lowercase(s: &str) -> Self {
        match s {
            "normal" => Self::Normal,
            "soft" => Self::Soft,
            "drum" => Self::Drum,
            _ => Self::None,
        }
    }
}

impl FromStr for SampleBank {
    type Err = ParseSampleBankError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" | "None" => Ok(Self::None),
            "1" | "Normal" => Ok(Self::Normal),
            "2" | "Soft" => Ok(Self::Soft),
            "3" | "Drum" => Ok(Self::Drum),
            _ => Err(ParseSampleBankError),
        }
    }
}

impl TryFrom<i32> for SampleBank {
    type Error = ParseSampleBankError;

    fn try_from(bank: i32) -> Result<Self, Self::Error> {
        match bank {
            0 => Ok(Self::None),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Soft),
            3 => Ok(Self::Drum),
            _ => Err(ParseSampleBankError),
        }
    }
}

/// Error type for a failed parsing of [`SampleBank`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid sample bank value")]
pub struct ParseSampleBankError;

/// The type of hit sample.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct HitSoundType(u8);

impl HitSoundType {
    pub const NONE: u8 = 0;
    pub const NORMAL: u8 = 1;
    pub const WHISTLE: u8 = 2;
    pub const FINISH: u8 = 4;
    pub const CLAP: u8 = 5;

    pub const fn has_flag(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }
}

impl From<&[HitSampleInfo]> for HitSoundType {
    fn from(samples: &[HitSampleInfo]) -> Self {
        let mut kind = Self::NONE;

        for sample in samples.iter() {
            match sample.name {
                Some(HitSampleInfoName::Whistle) => kind |= Self::WHISTLE,
                Some(HitSampleInfoName::Finish) => kind |= Self::FINISH,
                Some(HitSampleInfoName::Clap) => kind |= Self::CLAP,
                Some(HitSampleInfoName::Normal) | None => {}
            }
        }

        Self(kind)
    }
}

impl From<HitSoundType> for u8 {
    fn from(kind: HitSoundType) -> Self {
        kind.0
    }
}

impl FromStr for HitSoundType {
    type Err = ParseHitSoundTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self).map_err(ParseHitSoundTypeError)
    }
}

/// Error type for a failed parsing of [`HitSoundType`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid hit sound type")]
pub struct ParseHitSoundTypeError(#[source] ParseIntError);

impl PartialEq<u8> for HitSoundType {
    fn eq(&self, other: &u8) -> bool {
        self.0.eq(other)
    }
}

impl BitAnd<u8> for HitSoundType {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        self.0 & rhs
    }
}

impl BitAndAssign<u8> for HitSoundType {
    fn bitand_assign(&mut self, rhs: u8) {
        self.0 &= rhs;
    }
}

/// Sample info of a [`HitObject`] to convert [`HitSoundType`] into a [`Vec`]
/// of [`HitSampleInfo`].
/// 
/// [`HitObject`]: crate::model::hit_objects::HitObject
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SampleBankInfo {
    pub filename: Option<String>,
    pub bank_for_normal: Option<SampleBank>,
    pub bank_for_addition: Option<SampleBank>,
    pub volume: i32,
    pub custom_sample_bank: i32,
}

impl SampleBankInfo {
    /// Read and store custom sample banks.
    pub fn read_custom_sample_banks(
        &mut self,
        mut split: Split<'_, char>,
    ) -> Result<(), ParseSampleBankInfoError> {
        let Some(first) = split.next() else {
            return Ok(());
        };

        let bank = i32::parse(first)?.try_into().unwrap_or(SampleBank::Normal);

        let add_bank = split
            .next()
            .ok_or(ParseSampleBankInfoError::MissingInfo)?
            .parse_num::<i32>()?
            .try_into()
            .unwrap_or(SampleBank::Normal);

        let normal_bank = (bank != SampleBank::None).then_some(bank);
        let add_bank = (add_bank != SampleBank::None).then_some(add_bank);

        self.bank_for_normal = normal_bank;
        self.bank_for_addition = add_bank.or(normal_bank);

        if let Some(next) = split.next() {
            self.custom_sample_bank = next.parse_num()?;
        }

        if let Some(next) = split.next() {
            self.volume = cmp::max(0, next.parse_num()?);
        }

        self.filename = split.next().map(str::to_owned);

        Ok(())
    }

    /// Convert a [`HitSoundType`] into a [`Vec`] of [`HitSampleInfo`].
    pub fn convert_sound_type(self, sound_type: HitSoundType) -> Vec<HitSampleInfo> {
        let mut sound_types = Vec::new();

        if self.filename.as_ref().is_some_and(|s| !s.is_empty()) {
            sound_types.push(HitSampleInfo {
                filename: self.filename,
                ..HitSampleInfo::new(None, None, 1, self.volume)
            });
        } else {
            let mut sample = HitSampleInfo::new(
                Some(HitSampleInfoName::Normal),
                self.bank_for_normal,
                self.custom_sample_bank,
                self.volume,
            );

            sample.is_layered =
                sound_type != HitSoundType::NONE && !sound_type.has_flag(HitSoundType::NORMAL);

            sound_types.push(sample);
        }

        if sound_type.has_flag(HitSoundType::FINISH) {
            sound_types.push(HitSampleInfo::new(
                Some(HitSampleInfoName::Finish),
                self.bank_for_addition,
                self.custom_sample_bank,
                self.volume,
            ));
        }

        if sound_type.has_flag(HitSoundType::WHISTLE) {
            sound_types.push(HitSampleInfo::new(
                Some(HitSampleInfoName::Whistle),
                self.bank_for_addition,
                self.custom_sample_bank,
                self.volume,
            ));
        }

        if sound_type.has_flag(HitSoundType::CLAP) {
            sound_types.push(HitSampleInfo::new(
                Some(HitSampleInfoName::Clap),
                self.bank_for_addition,
                self.custom_sample_bank,
                self.volume,
            ));
        }

        sound_types
    }
}

/// All the ways that parsing into [`SampleBankInfo`] can fail.
#[derive(Debug, thiserror::Error)]
pub enum ParseSampleBankInfoError {
    #[error("missing info")]
    MissingInfo,
    #[error("failed to parse number")]
    Number(#[from] ParseNumberError),
}
