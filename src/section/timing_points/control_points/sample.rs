use std::cmp::Ordering;

use crate::{section::hit_objects::hit_samples::SampleBank, util::Sortable};

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
        Some(<Self as Sortable>::cmp(self, other))
    }
}

impl Sortable for SamplePoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.total_cmp(&other.time)
    }
}
