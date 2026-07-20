# srs-sm2

[![crates.io](https://img.shields.io/crates/v/srs-sm2.svg)](https://crates.io/crates/srs-sm2)
[![docs.rs](https://docs.rs/srs-sm2/badge.svg)](https://docs.rs/srs-sm2)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

Pure, lightweight, `#![no_std]`-compatible implementation of the **SuperMemo
SM-2** spaced-repetition scheduling algorithm — the classic algorithm behind
flashcard apps.

```toml
[dependencies]
srs-sm2 = "0.1"
```

> Note: the name `srs-sm2` refers to **S**paced **R**epetition **S**ystem using
> the **S**uper**M**emo **2** algorithm. It is unrelated to the SM2 elliptic
> curve used in cryptography.

## Why this crate?

* **Pure & side-effect free** — `schedule_next` takes a [`Schedule`] and a
  recall quality, and returns the next [`Schedule`]. No clock, no network, no
  storage.
* **`#![no_std]` compatible** — the core logic needs no `std`; it relies only on
  [`libm`] for floating-point math, so it compiles for WASM, embedded, or
  anywhere else.
* **Small & auditable** — a single constant and one function. Easy to read,
  easy to trust, easy to vendor.
* **Battle-tested in spirit** — the algorithm is the well-documented SM-2 spec,
  and the crate ships unit tests pinning every branch of the formula.

## Design goals

* **Pure function** — same input, same output, forever.
* **Deterministic** — no randomness, no hidden state.
* **`no_std` compatible** — runs where `std` does not.
* **Framework agnostic** — no Leptos, no React, no async runtime assumptions.
* **No storage assumptions** — you keep the schedule; this crate only computes it.
* **No clock access** — time is the caller's concern, not ours.
* **Tiny API surface** — one type, one function, one constant.

## Scope: why SM-2 (and not FSRS)

This crate intentionally implements the **original SuperMemo SM-2 algorithm**.
It does not attempt to implement newer scheduling systems such as **FSRS** or
later SuperMemo variants. The goal is a small, deterministic implementation of
the classic algorithm suitable for education, embedded systems, and
applications that require compatibility with existing SM-2 schedules.

If you need adaptive, ML-based scheduling, FSRS is the better tool. If you want
a footprint you can read in one sitting and ship to a microcontroller, this is
it.

## Usage

```rust
use srs_sm2::{Schedule, schedule_next, MIN_EASE_FACTOR};

// A brand-new card, never reviewed.
let card = Schedule { interval_days: 0, ease_factor: 2.5 };

// Perfect recall (quality 5) seeds the first interval to 1 day.
let next = schedule_next(card, 5);
assert_eq!(next.interval_days, 1);

// A second good review seeds 6 days.
let next = schedule_next(next, 4);
assert_eq!(next.interval_days, 6);

// A continued perfect streak grows the interval by the ease factor.
let next = schedule_next(next, 5);
assert_eq!(next.interval_days, 16); // 6 * 2.6, rounded
assert!(next.ease_factor >= MIN_EASE_FACTOR);
```

### Quality scale

`schedule_next` takes a `quality` in `0..=5` (matching the original SM-2
grading):

| Quality | Meaning                              |
|---------|--------------------------------------|
| 0–2     | Failed recall — interval resets to 1 |
| 3       | Recalled with serious difficulty     |
| 4       | Recalled with some hesitation        |
| 5       | Perfect recall                       |

Values outside `0..=5` are clamped, so you can pass slider or keyboard input
directly.

## Framework-agnostic examples

Because the crate is just a pure function, it drops into any context.

### Leptos / WASM

```rust
use leptos::*;
use srs_sm2::schedule_next;

#[component]
fn flashcard(mut card: Signal<Schedule>, quality: Signal<i32>) -> impl IntoView {
    let next = Signal::derive(move || schedule_next(card.get(), quality.get()));
    view! { <p>"Review again in " {move || next.get().interval_days} " days"</p> }
}
```

### CLI

```rust
use srs_sm2::{Schedule, schedule_next};

fn main() {
    let card = Schedule { interval_days: 0, ease_factor: 2.5 };
    let rating = 5; // from argv / prompt
    let due = schedule_next(card, rating).interval_days;
    println!("Review in {due} days");
}
```

### Embedded / `no_std`

```toml
[dependencies]
srs-sm2 = { version = "0.1", default-features = false }
```

```rust
#![no_std]
use srs_sm2::{Schedule, schedule_next};

// Store `Schedule` in your flash/EEPROM; compute the next review on the fly.
let card = Schedule { interval_days: 6, ease_factor: 2.5 };
let next = schedule_next(card, 4);
```

## Algorithm

For each review, with recall quality `q` in `0..=5`:

```text
EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))   # clamped to >= 1.3
```

The next interval is then:

* `quality < 3` or first review → `1` day
* second review → `6` days
* otherwise → `round(interval * EF')` days

## Minimum Supported Rust Version

This crate supports Rust **1.70** and later.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
