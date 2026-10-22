//! Throughput from cumulative bytes over a bounded observation window.
//! A restored counter establishes the baseline, never new transferred bytes.
use std::collections::VecDeque;

#[derive(Default)]
pub struct TransferSpeed {
    samples: VecDeque<(u64, u64)>,
}

impl TransferSpeed {
    pub fn sample(&mut self, now_ms: u64, bytes: u64) -> u64 {
        if self
            .samples
            .back()
            .is_some_and(|&(time, count)| now_ms < time || bytes < count)
        {
            self.samples.clear();
        }
        if self.samples.back().is_some_and(|&(time, _)| time == now_ms) {
            return 0;
        }
        self.samples.push_back((now_ms, bytes));
        let cutoff = now_ms.saturating_sub(2_000);
        while self.samples.len() > 2 && self.samples[1].0 <= cutoff {
            self.samples.pop_front();
        }
        let &(start, initial) = self.samples.front().unwrap();
        let elapsed = now_ms.saturating_sub(start);
        if elapsed == 0 {
            0
        } else {
            ((u128::from(bytes.saturating_sub(initial)) * 1_000) / u128::from(elapsed))
                .min(u128::from(u64::MAX)) as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bursty_block_updates_measure_sustained_throughput_and_then_a_real_stall() {
        let mut speed = TransferSpeed::default();
        speed.sample(0, 0);
        for step in 1..=16 {
            let bytes = (step / 2) * 50_000_000;
            let rate = speed.sample(step * 250, bytes);
            if step >= 8 {
                assert_eq!(rate, 100_000_000);
            }
        }
        // The network really stops: the bounded window must reach zero.
        for step in 17..=24 {
            speed.sample(step * 250, 400_000_000);
        }
        assert_eq!(speed.sample(6_250, 400_000_000), 0);
    }

    #[test]
    fn resumed_bytes_and_counter_resets_never_become_a_speed_spike() {
        let mut speed = TransferSpeed::default();
        assert_eq!(speed.sample(0, 5_000_000_000), 0);
        assert_eq!(speed.sample(1_000, 5_100_000_000), 100_000_000);
        assert_eq!(speed.sample(1_100, 0), 0);
        assert_eq!(speed.sample(2_100, 100_000_000), 100_000_000);
    }
}
