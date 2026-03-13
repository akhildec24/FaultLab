//! Traffic generator — converts a `TrafficConfig` into `RequestCreated`
//! events on the scheduler.
//!
//! The generator models a linear ramp from `start_rps` to `target_rps`
//! over `ramp_seconds`. After the ramp, traffic stays at `target_rps`.
//!
//! For each second of simulation time, the generator calculates the
//! current rps, then schedules that many `RequestCreated` events evenly
//! spaced within the second.

use crate::rng::Rng;
use crate::scheduler::Scheduler;
use crate::types::*;

/// The traffic generator.
pub struct TrafficGenerator {
    config: TrafficConfig,
    client_ids: Vec<String>,
    /// Total simulation duration in milliseconds (0 = run until stopped).
    duration_ms: u64,
}

impl TrafficGenerator {
    pub fn new(config: TrafficConfig, client_ids: Vec<String>) -> Self {
        Self {
            config,
            client_ids,
            duration_ms: 0,
        }
    }

    /// Set a maximum simulation duration in milliseconds.
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Calculate the current rps at a given virtual time.
    pub fn rps_at(&self, time_ms: u64) -> f64 {
        let ramp_ms = self.config.ramp_seconds * 1000;
        if ramp_ms == 0 {
            return self.config.target_rps as f64;
        }
        let progress = (time_ms as f64 / ramp_ms as f64).min(1.0);
        let start = self.config.start_rps as f64;
        let target = self.config.target_rps as f64;
        start + (target - start) * progress
    }

    /// Schedule all traffic events onto the scheduler.
    ///
    /// Generates `RequestCreated` events for each second of the simulation.
    /// If `duration_ms` is 0, defaults to ramp_seconds + 30 seconds.
    pub fn schedule_all(
        &self,
        scheduler: &mut Scheduler,
        rng: &mut Rng,
        next_request_id: &mut u64,
    ) {
        let duration = if self.duration_ms > 0 {
            self.duration_ms
        } else {
            (self.config.ramp_seconds + 30) * 1000
        };

        let mut time = 0u64;
        while time < duration {
            let rps = self.rps_at(time);
            let count = rps.round() as u64;
            let interval = 1000u64.checked_div(count).unwrap_or(1000);

            for i in 0..count {
                let event_time = time + i * interval;
                if event_time >= duration {
                    break;
                }
                let client = if self.client_ids.len() == 1 {
                    self.client_ids[0].clone()
                } else {
                    self.client_ids[rng.below(self.client_ids.len() as u64) as usize].clone()
                };
                let id = *next_request_id;
                *next_request_id += 1;
                scheduler.schedule(
                    VirtualTime(event_time),
                    Event::RequestCreated {
                        request_id: id,
                        origin: client,
                    },
                );
            }
            time += 1000;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_at_start_is_start_rps() {
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 20,
                target_rps: 500,
                ramp_seconds: 30,
            },
            vec!["client".into()],
        );
        assert_eq!(gen.rps_at(0), 20.0);
    }

    #[test]
    fn rps_at_end_of_ramp_is_target() {
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 20,
                target_rps: 500,
                ramp_seconds: 30,
            },
            vec!["client".into()],
        );
        assert_eq!(gen.rps_at(30_000), 500.0);
    }

    #[test]
    fn rps_at_midpoint_is_average() {
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 20,
                target_rps: 500,
                ramp_seconds: 30,
            },
            vec!["client".into()],
        );
        let mid = gen.rps_at(15_000);
        assert!((mid - 260.0).abs() < 0.1);
    }

    #[test]
    fn rps_after_ramp_stays_at_target() {
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 20,
                target_rps: 500,
                ramp_seconds: 30,
            },
            vec!["client".into()],
        );
        assert_eq!(gen.rps_at(60_000), 500.0);
    }

    #[test]
    fn rps_zero_ramp_uses_target() {
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 10,
                target_rps: 100,
                ramp_seconds: 0,
            },
            vec!["client".into()],
        );
        assert_eq!(gen.rps_at(0), 100.0);
        assert_eq!(gen.rps_at(5000), 100.0);
    }

    #[test]
    fn schedule_all_creates_events() {
        let mut scheduler = Scheduler::new();
        let mut rng = Rng::new(42);
        let mut next_id = 1u64;
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 5,
                target_rps: 5,
                ramp_seconds: 0,
            },
            vec!["client".into()],
        )
        .with_duration(3000);

        gen.schedule_all(&mut scheduler, &mut rng, &mut next_id);

        let mut count = 0;
        while scheduler.next_event().is_some() {
            count += 1;
        }
        // 5 rps for 3 seconds = 15 requests
        assert_eq!(count, 15);
        assert_eq!(next_id, 16);
    }

    #[test]
    fn schedule_all_with_ramp_increases_requests() {
        let mut scheduler = Scheduler::new();
        let mut rng = Rng::new(42);
        let mut next_id = 1u64;
        let gen = TrafficGenerator::new(
            TrafficConfig {
                start_rps: 10,
                target_rps: 100,
                ramp_seconds: 2,
            },
            vec!["client".into()],
        )
        .with_duration(3000);

        gen.schedule_all(&mut scheduler, &mut rng, &mut next_id);

        // Count events in each second
        let mut per_second = [0u32; 3];
        while let Some((t, _)) = scheduler.next_event() {
            let sec = (t.millis() / 1000) as usize;
            if sec < 3 {
                per_second[sec] += 1;
            }
        }
        // Second 0: ~10 rps, second 1: ~55 rps, second 2: ~100 rps
        assert!(per_second[0] <= 15, "second 0 had {} events", per_second[0]);
        assert!(per_second[2] >= 90, "second 2 had {} events", per_second[2]);
    }
}
