use std::{cmp::Ordering, num::NonZeroU32};

use crate::section::hit_objects::hit_samples::{HitSampleInfo, SampleBank};

#[derive(Clone, Debug, PartialEq)]
pub struct SamplePoint {
    pub time: f64,
    pub sample_bank: SampleBank,
    pub sample_volume: i32,
    pub custom_sample_bank: i32,
}

impl SamplePoint {
    pub const DEFAULT_SAMPLE_BANK: SampleBank = SampleBank::Normal;
    pub const DEFAULT_SAMPLE_VOLUME: i32 = 100;
    pub const DEFAULT_CUSTOM_SAMPLE_BANK: i32 = 0;

    pub const fn new(
        time: f64,
        sample_bank: SampleBank,
        sample_volume: i32,
        custom_sample_bank: i32,
    ) -> Self {
        Self {
            time,
            sample_bank,
            sample_volume,
            custom_sample_bank,
        }
    }

    pub fn is_redundant(&self, existing: &Self) -> bool {
        self.sample_bank == existing.sample_bank
            && self.sample_volume == existing.sample_volume
            && self.custom_sample_bank == existing.custom_sample_bank
    }

    pub fn apply(&self, sample: &mut HitSampleInfo) {
        if sample.custom_sample_bank == 0 {
            sample.custom_sample_bank = self.custom_sample_bank;

            if sample.custom_sample_bank >= 2 {
                // SAFETY: The value is guaranteed to be >= 2
                sample.suffix =
                    Some(unsafe { NonZeroU32::new_unchecked(sample.custom_sample_bank as u32) });
            }
        }

        if sample.volume == 0 {
            sample.volume = self.sample_volume;
        }

        if !sample.bank_specified {
            sample.bank = self.sample_bank;
            sample.bank_specified = true;
        }
    }
}

impl Default for SamplePoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            sample_bank: Self::DEFAULT_SAMPLE_BANK,
            sample_volume: Self::DEFAULT_SAMPLE_VOLUME,
            custom_sample_bank: Self::DEFAULT_CUSTOM_SAMPLE_BANK,
        }
    }
}

impl PartialOrd for SamplePoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
