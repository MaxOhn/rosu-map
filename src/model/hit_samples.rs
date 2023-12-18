use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitSampleInfo {
    pub name: HitSampleInfoName,
    pub bank: &'static str,
    pub suffix: Option<String>,
    pub volume: i32,
    pub custom_sample_bank: i32,
    pub bank_specified: bool,
    pub is_layered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HitSampleInfoName {
    Name(&'static str),
    File(Option<String>),
}

impl From<&'static str> for HitSampleInfoName {
    fn from(name: &'static str) -> Self {
        Self::Name(name)
    }
}

impl From<Option<String>> for HitSampleInfoName {
    fn from(filename: Option<String>) -> Self {
        Self::File(filename)
    }
}

impl HitSampleInfo {
    pub const HIT_NORMAL: &'static str = "hitnormal";
    pub const HIT_WHISTLE: &'static str = "hitwhistle";
    pub const HIT_FINISH: &'static str = "hitfinish";
    pub const HIT_CLAP: &'static str = "hitclap";

    pub const BANK_NORMAL: &'static str = "normal";
    pub const BANK_SOFT: &'static str = "soft";
    pub const BANK_DRUM: &'static str = "drum";

    pub fn new(
        name: impl Into<HitSampleInfoName>,
        bank: Option<&'static str>,
        custom_sample_bank: i32,
        volume: i32,
    ) -> Self {
        Self {
            name: name.into(),
            bank: bank.unwrap_or(Self::BANK_NORMAL),
            suffix: (custom_sample_bank >= 2).then(|| custom_sample_bank.to_string()),
            volume,
            custom_sample_bank,
            bank_specified: bank.is_some(),
            is_layered: false,
        }
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

impl SampleBank {
    pub const fn to_lowercase_str(self) -> &'static str {
        match self {
            SampleBank::None => "none",
            SampleBank::Normal => "normal",
            SampleBank::Soft => "soft",
            SampleBank::Drum => "drum",
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
