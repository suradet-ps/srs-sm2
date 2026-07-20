//! Pure implementation of the SuperMemo SM-2 spaced-repetition algorithm.
//!
//! `srs-sm2` is a tiny, lightweight, `#![no_std]`-compatible library that
//! computes *when a flashcard should be reviewed next*. Given a card's current
//! schedule and the quality of a recall, it returns the next schedule — with no
//! I/O, no allocations beyond what the caller supplies, and no clock access.
//! That makes it equally at home in a browser WASM bundle, an embedded device,
//! or a server.
//!
//! # The algorithm
//!
//! SM-2 tracks two numbers per card:
//!
//! * `interval_days` — days until the next review.
//! * `ease_factor` — a multiplier (`>=` [`MIN_EASE_FACTOR`]) controlling how
//!   fast the interval grows.
//!
//! On each review the caller supplies a `quality` in `0..=5`:
//!
//! * `quality < 3` is a failed recall: the interval resets to 1 day (the card is
//!   due again tomorrow) but the ease factor is still updated.
//! * The first successful review seeds the interval to 1 day, the second to 6
//!   days, and every review after that multiplies the previous interval by the
//!   (updated) ease factor.
//!
//! The ease factor is updated every review with the classic SM-2 formula:
//!
//! ```text
//! EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
//! ```
//!
//! clamped to never drop below [`MIN_EASE_FACTOR`] (1.3).
//!
//! # Cargo features
//!
//! * `std` (default) — enables `std`-only helpers. The core scheduling logic
//!   does not need it; disable default features for a `no_std` build.
//!
//! # Examples
//!
//! ```
//! use srs_sm2::{Schedule, schedule_next, MIN_EASE_FACTOR};
//!
//! // A brand-new card, never reviewed.
//! let card = Schedule { interval_days: 0, ease_factor: 2.5 };
//!
//! // Perfect recall (quality 5) → first interval is seeded to 1 day.
//! let next = schedule_next(card, 5);
//! assert_eq!(next.interval_days, 1);
//!
//! // A second good review seeds 6 days.
//! let next = schedule_next(next, 4);
//! assert_eq!(next.interval_days, 6);
//!
//! // A continued perfect streak grows the interval by the ease factor.
//! let next = schedule_next(next, 5);
//! assert_eq!(next.interval_days, 16); // 6 * 2.6, rounded
//! assert!(next.ease_factor >= MIN_EASE_FACTOR);
//! ```
//!
//! A failed recall (`quality < 3`) resets the interval without dropping the
//! ease factor below the floor:
//!
//! ```
//! use srs_sm2::{Schedule, schedule_next, MIN_EASE_FACTOR};
//!
//! let card = Schedule { interval_days: 30, ease_factor: 2.5 };
//! let failed = schedule_next(card, 2);
//! assert_eq!(failed.interval_days, 1);
//! assert!(failed.ease_factor >= MIN_EASE_FACTOR);
//! ```

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

extern crate alloc;

use libm::{fmax, round};

/// The lowest an ease factor may fall. Below this, intervals collapse toward
/// daily review; SM-2 fixes the floor at 1.3.
pub const MIN_EASE_FACTOR: f64 = 1.3;

/// A card's schedule state, independent of storage or rendering.
///
/// `Schedule` is plain data: it carries the two numbers SM-2 needs and nothing
/// else. Clone and copy it freely; the library never reaches back into your
/// data structures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Schedule {
    /// Days until the next review.
    pub interval_days: i32,
    /// `SuperMemo` ease factor (`>=` [`MIN_EASE_FACTOR`]).
    pub ease_factor: f64,
}

/// Compute the next [`Schedule`] from the current one and a recall `quality`
/// in `0..=5`.
///
/// Values outside `0..=5` are clamped to the nearest endpoint, so callers do
/// not need to pre-validate slider/keyboard input.
///
/// * `quality < 3` is treated as a failed recall: the interval resets to 1 day.
/// * The ease factor is always updated and clamped to [`MIN_EASE_FACTOR`].
///
/// # Examples
///
/// ```
/// use srs_sm2::{Schedule, schedule_next};
///
/// // First successful review seeds a 1-day interval.
/// let next = schedule_next(Schedule { interval_days: 0, ease_factor: 2.5 }, 5);
/// assert_eq!(next.interval_days, 1);
///
/// // Out-of-range quality is clamped (99 behaves like 5).
/// let hi = schedule_next(Schedule { interval_days: 6, ease_factor: 2.5 }, 99);
/// let five = schedule_next(Schedule { interval_days: 6, ease_factor: 2.5 }, 5);
/// assert_eq!(hi, five);
/// ```
#[must_use]
pub fn schedule_next(current: Schedule, quality: i32) -> Schedule {
    let q = f64::from(quality.clamp(0, 5));
    let diff = 5.0 - q;
    // EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
    let delta = 0.1 - diff * (0.08 + diff * 0.02);
    let ease_factor = fmax(current.ease_factor + delta, MIN_EASE_FACTOR);

    let interval_days = if quality < 3 || current.interval_days == 0 {
        1
    } else if current.interval_days == 1 {
        6
    } else {
        let raw = f64::from(current.interval_days) * ease_factor;
        round(raw) as i32
    };

    Schedule {
        interval_days,
        ease_factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn failed_recall_resets_interval() {
        let next = schedule_next(
            Schedule {
                interval_days: 30,
                ease_factor: 2.5,
            },
            2,
        );
        assert_eq!(next.interval_days, 1, "failed recall must reset interval to 1");
    }

    #[test]
    fn first_success_seeds_one_day() {
        let next = schedule_next(
            Schedule {
                interval_days: 0,
                ease_factor: 2.5,
            },
            5,
        );
        assert_eq!(next.interval_days, 1);
    }

    #[test]
    fn second_success_seeds_six_days() {
        let next = schedule_next(
            Schedule {
                interval_days: 1,
                ease_factor: 2.5,
            },
            4,
        );
        assert_eq!(next.interval_days, 6);
    }

    #[test]
    fn subsequent_success_multiplies_by_ease() {
        // interval 6, perfect recall keeps EF at 2.6 -> 6 * 2.6 = 15.6 -> 16
        let next = schedule_next(
            Schedule {
                interval_days: 6,
                ease_factor: 2.5,
            },
            5,
        );
        assert!(approx(next.ease_factor, 2.6), "ease was {}", next.ease_factor);
        assert_eq!(next.interval_days, 16);
    }

    #[test]
    fn ease_factor_never_below_floor() {
        let mut sched = Schedule {
            interval_days: 10,
            ease_factor: 1.3,
        };
        for _ in 0..20 {
            sched = schedule_next(sched, 0);
            assert!(
                sched.ease_factor >= MIN_EASE_FACTOR,
                "ease dropped below floor: {}",
                sched.ease_factor
            );
        }
    }

    #[test]
    fn perfect_recall_raises_ease() {
        let next = schedule_next(
            Schedule {
                interval_days: 6,
                ease_factor: 2.5,
            },
            5,
        );
        assert!(next.ease_factor > 2.5, "perfect recall should raise ease");
    }

    #[test]
    fn quality_three_holds_ease_roughly_steady() {
        // q=3: delta = 0.1 - 2*(0.08 + 2*0.02) = 0.1 - 2*0.12 = -0.14
        let next = schedule_next(
            Schedule {
                interval_days: 6,
                ease_factor: 2.5,
            },
            3,
        );
        assert!(approx(next.ease_factor, 2.36), "ease was {}", next.ease_factor);
    }

    #[test]
    fn out_of_range_quality_is_clamped() {
        let hi = schedule_next(
            Schedule {
                interval_days: 6,
                ease_factor: 2.5,
            },
            99,
        );
        let five = schedule_next(
            Schedule {
                interval_days: 6,
                ease_factor: 2.5,
            },
            5,
        );
        assert_eq!(hi, five, "quality above 5 should clamp to 5");
    }
}
