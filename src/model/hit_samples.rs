use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitSampleInfo {
    pub name: Option<HitSampleInfoName>,
    pub filename: Option<String>,
    pub bank: SampleBank,
    pub suffix: Option<i32>,
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
            suffix: (custom_sample_bank >= 2).then_some(custom_sample_bank),
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid sample bank value")]
pub struct ParseSampleBankError;
