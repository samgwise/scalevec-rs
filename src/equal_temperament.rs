// EqualTemperament: maps step values directly to frequencies.
// Ported from ScaleVec::Scale::EqualTemperment in p6-ScaleVec.

use super::Scale;

/// Maps step indices to frequencies using equal temperament.
///
/// The formula is: `ref_freq * 2^((step - ref_step) / tones_per_octave)`
///
/// With default settings (A4 = 440 Hz, step 49, 12 tones/octave), this
/// produces standard 12-TET frequencies. But it works with any number of
/// tones per octave for microtonal equal temperaments.
#[derive(Debug, Clone)]
pub struct EqualTemperament {
    /// Number of equal divisions of the octave.
    pub tones_per_octave: f64,
    /// Reference frequency in Hz (default: 440.0 for A4).
    pub ref_freq: f64,
    /// The step index that corresponds to the reference frequency.
    pub ref_step: f64,
}

impl Default for EqualTemperament {
    fn default() -> Self {
        EqualTemperament {
            tones_per_octave: 12.0,
            ref_freq: 440.0,
            ref_step: 49.0,
        }
    }
}

impl EqualTemperament {
    pub fn new(tones_per_octave: f64, ref_freq: f64, ref_step: f64) -> Self {
        EqualTemperament {
            tones_per_octave,
            ref_freq,
            ref_step,
        }
    }

    /// The frequency ratio for one step.
    pub fn root_of_two(&self) -> f64 {
        2.0_f64.powf(1.0 / self.tones_per_octave)
    }
}

impl Scale for EqualTemperament {
    fn step(&self, step: f64) -> f64 {
        self.ref_freq * self.root_of_two().powf(step - self.ref_step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 0.01;

    #[test]
    fn test_reference_pitch() {
        let et = EqualTemperament::default();
        // Step 49 should give A4 = 440 Hz
        let freq = et.step(49.0);
        assert!((freq - 440.0).abs() < EPSILON, "got {}", freq);
    }

    #[test]
    fn test_octave_above() {
        let et = EqualTemperament::default();
        // Step 61 (49 + 12) should give A5 = 880 Hz
        let freq = et.step(61.0);
        assert!((freq - 880.0).abs() < EPSILON, "got {}", freq);
    }

    #[test]
    fn test_octave_below() {
        let et = EqualTemperament::default();
        // Step 37 (49 - 12) should give A3 = 220 Hz
        let freq = et.step(37.0);
        assert!((freq - 220.0).abs() < EPSILON, "got {}", freq);
    }

    #[test]
    fn test_middle_c() {
        let et = EqualTemperament::default();
        // Middle C is step 40 (49 - 9 semitones below A4)
        // C4 ≈ 261.63 Hz
        let freq = et.step(40.0);
        assert!((freq - 261.63).abs() < 0.1, "got {}", freq);
    }

    #[test]
    fn test_19_tet() {
        // 19-tone equal temperament
        let et = EqualTemperament::new(19.0, 440.0, 49.0);
        // One octave above reference
        let freq = et.step(49.0 + 19.0);
        assert!((freq - 880.0).abs() < EPSILON, "got {}", freq);
    }
}
