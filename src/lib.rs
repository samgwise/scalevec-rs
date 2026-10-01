//! ScaleVec: a flexible music representation system.
//!
//! Ported from [p6-ScaleVec](https://github.com/samgwise/p6-ScaleVec).
//!
//! ScaleVec represents scales, chords, rhythms, or any ordered numeric
//! structure as a vector of values with automatic period-wrapping and
//! fractional interpolation. Types implement the [`Scale`] trait, mapping a
//! step index to a value, so they compose into pipelines:
//!
//! ```
//! use scalevec::{EqualTemperament, Scale, ScaleVec, Stack};
//!
//! // A major scale in semitones
//! let major = ScaleVec::new(vec![0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0], None);
//!
//! // Map scale degrees to frequencies via equal temperament
//! let et = EqualTemperament::new(12.0, 261.63, 0.0);
//! let stack = Stack::new(vec![Box::new(major), Box::new(et)]);
//!
//! // Degree 4 (G) is about 392 Hz
//! assert!((stack.step(4.0) - 392.0).abs() < 1.0);
//! ```

mod core;
mod equal_temperament;
mod fence;
mod scl;
mod stack;

pub use self::core::ScaleVec;
pub use self::equal_temperament::EqualTemperament;
pub use self::fence::Fence;
pub use self::scl::{from_scl, parse_custom_scale, parse_interval, to_scl};
pub use self::stack::Stack;

/// Trait for anything that can map a step index to a value.
/// This is the universal interface shared by ScaleVec, Fence,
/// EqualTemperament, and Stack.
pub trait Scale {
    /// Map an abstract step index to a concrete value.
    fn step(&self, n: f64) -> f64;
}
