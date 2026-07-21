# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-07-21

### Fixed

- Correct repository URL in Cargo.toml (`vate-ps` → `suradet-ps`)

## [0.1.0] - 2026-07-21

### Added

- `Schedule` struct with `interval_days` and `ease_factor` fields
- `Default` implementation for `Schedule` (zero interval, 2.5 ease factor)
- `schedule_next()` function — computes the next review schedule from quality (0..=5)
- `MIN_EASE_FACTOR` constant (1.3) — floor for ease factor clamping
- `#![no_std]` support via `libm` for math functions
- Optional `serde` feature for serialization (Serialize / Deserialize)
- `missing_docs = "deny"` lint
- CI workflow with format check, clippy, tests, no_std build, and security audit
- Dual MIT / Apache-2.0 license
