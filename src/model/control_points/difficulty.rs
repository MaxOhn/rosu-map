use std::cmp::Ordering;

use crate::util::{Sortable, SortedVec};

#[derive(Clone, Debug, PartialEq)]
pub struct DifficultyPoint {
    pub time: f64,
    pub bpm_multiplier: f64,
    pub slider_velocity: f64,
    pub generate_ticks: bool,
}

impl DifficultyPoint {
    pub const DEFAULT_SLIDER_VELOCITY: f64 = 1.0;
    pub const DEFAULT_BPM_MULTIPLIER: f64 = 1.0;
    pub const DEFAULT_GENERATE_TICKS: bool = true;

    pub fn new(time: f64, beat_len: f64, speed_multiplier: f64) -> Self {
        let bpm_multiplier = if beat_len < 0.0 {
            f64::from(((-beat_len) as f32).clamp(10.0, 10_000.0)) / 100.0
        } else {
            1.0
        };

        Self {
            time,
            slider_velocity: speed_multiplier.clamp(0.1, 10.0),
            bpm_multiplier,
            generate_ticks: !beat_len.is_nan(),
        }
    }
}

impl Default for DifficultyPoint {
    fn default() -> Self {
        Self {
            time: 0.0,
            bpm_multiplier: Self::DEFAULT_BPM_MULTIPLIER,
            slider_velocity: Self::DEFAULT_SLIDER_VELOCITY,
            generate_ticks: Self::DEFAULT_GENERATE_TICKS,
        }
    }
}

impl PartialOrd for DifficultyPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(<Self as Sortable>::cmp(self, other))
    }
}

impl Sortable for DifficultyPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.total_cmp(&other.time)
    }

    fn is_redundant(&self, existing: &Self) -> bool {
        (self.slider_velocity - existing.slider_velocity).abs() < f64::EPSILON
            && self.generate_ticks == existing.generate_ticks
    }

    fn push(self, sorted_vec: &mut SortedVec<Self>) {
        enum Action {
            Insert(usize),
            Replace(usize),
            Push,
            Skip,
        }

        let action = match sorted_vec.find(&self).map_err(|idx| idx.checked_sub(1)) {
            Ok(i) | Err(Some(i)) if self.is_redundant(&sorted_vec[i]) => Action::Skip,
            Ok(i) => Action::Replace(i),
            Err(Some(i)) if i == sorted_vec.len() - 1 => Action::Push,
            Err(Some(i)) => Action::Insert(i),
            Err(None) if self.is_redundant(&Self::default()) => Action::Skip,
            Err(None) => Action::Insert(0),
        };

        // SAFETY: Items are inserted based on `<DifficultyPoint as Sortable>::cmp`
        //         which provides a valid ordering.
        let sorted_vec = unsafe { sorted_vec.as_inner_mut() };

        match action {
            Action::Insert(i) => sorted_vec.insert(i, self),
            Action::Replace(i) => sorted_vec[i] = self,
            Action::Push => sorted_vec.push(self),
            Action::Skip => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SortedVec;

    use super::*;

    #[test]
    fn no_push_if_redundant() {
        let mut v = SortedVec::default();

        v.push(DifficultyPoint::default());
        assert_eq!(v.len(), 0);

        v.push(DifficultyPoint::new(1.0, 2.0, 3.0));
        assert_eq!(v.len(), 1);

        v.push(DifficultyPoint::new(2.0, 2.0, 3.0));
        v.push(DifficultyPoint::default());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn from_iter() {
        let base = vec![
            DifficultyPoint {
                time: 5.0,
                slider_velocity: 10.0,
                bpm_multiplier: 1.0,
                generate_ticks: true,
            },
            DifficultyPoint {
                time: 3.0,
                slider_velocity: 20.0,
                bpm_multiplier: 2.0,
                generate_ticks: false,
            },
            DifficultyPoint {
                time: 6.0,
                slider_velocity: 10.0,
                bpm_multiplier: 3.0,
                generate_ticks: true,
            },
            DifficultyPoint {
                time: 10.0,
                slider_velocity: 15.0,
                bpm_multiplier: 4.0,
                generate_ticks: true,
            },
        ];

        let sorted = SortedVec::from_iter(base);

        let v: Vec<_> = sorted
            .into_inner()
            .into_iter()
            .map(|tp| tp.bpm_multiplier)
            .collect();

        assert_eq!(v, vec![2.0, 1.0, 4.0]);
    }
}
