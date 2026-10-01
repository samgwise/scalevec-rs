# scalevec

A flexible music representation system, ported from
[p6-ScaleVec](https://github.com/samgwise/p6-ScaleVec) (Raku) to Rust.

ScaleVec represents scales, chords, rhythms, or any ordered numeric structure
as a vector of values with automatic period-wrapping and fractional
interpolation. Types implement the `Scale` trait, mapping a step index to a
value, so they compose into pipelines:

```rust
use scalevec::{EqualTemperament, Scale, ScaleVec, Stack};

// A major scale in semitones
let major = ScaleVec::new(vec![0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0], None);

// Map scale degrees to frequencies via equal temperament
let et = EqualTemperament::new(12.0, 261.63, 0.0);
let stack = Stack::new(vec![Box::new(major), Box::new(et)]);

// Degree 4 (G) is about 392 Hz
assert!((stack.step(4.0) - 392.0).abs() < 1.0);
```

## Components

- `ScaleVec` — the core period-wrapping vector with transforms (transpose,
  retrograde, mirror inversion, augmentation/diminution, mapping, distance).
- `Fence` — clamps/wraps values within bounds (e.g. MIDI note range,
  pitch-class space).
- `EqualTemperament` — maps step indices to frequencies for any
  N-tones-per-octave temperament.
- `Stack` — composes a chain of `Scale`s into a pipeline.
- `scl` — Scala `.scl` file import/export and interval parsing
  (cents, ratios, and `n\m` EDO notation).

## Status

Pre-release. The API is still settling ahead of a first crates.io publish.
