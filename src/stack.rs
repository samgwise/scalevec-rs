// Stack: composes a chain of scales by reducing through step().
// Ported from ScaleVec::Scale::Stack in p6-ScaleVec.

use super::Scale;

/// Composes multiple Scale implementations into a pipeline.
///
/// The input is passed through each scale's `step()` in order:
/// `result = scales[n].step(scales[n-1].step(...scales[0].step(input)))`.
///
/// This is the core composition mechanism: e.g.
/// `degree → ScaleVec → Fence → EqualTemperament → frequency`
pub struct Stack {
    scales: Vec<Box<dyn Scale>>,
}

impl Stack {
    pub fn new(scales: Vec<Box<dyn Scale>>) -> Self {
        Stack { scales }
    }
}

impl Scale for Stack {
    fn step(&self, n: f64) -> f64 {
        self.scales.iter().fold(n, |value, scale| scale.step(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScaleVec, EqualTemperament};

    const EPSILON: f64 = 0.1;

    #[test]
    fn test_single_scale_stack() {
        // A Stack of one ScaleVec should behave identically to that ScaleVec
        let sv = ScaleVec::new(vec![0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0], None);
        let stack = Stack::new(vec![Box::new(sv.clone())]);

        for i in 0..14 {
            let direct = sv.step(i as f64);
            let stacked = stack.step(i as f64);
            assert!(
                (direct - stacked).abs() < f64::EPSILON,
                "step {}: direct={}, stacked={}",
                i,
                direct,
                stacked
            );
        }
    }

    #[test]
    fn test_scale_to_frequency_pipeline() {
        // Major scale → EqualTemperament (step 0 = C4)
        let major = ScaleVec::new(vec![0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0], None);
        // ET where step 0 = middle C ≈ 261.63 Hz
        let et = EqualTemperament::new(12.0, 261.63, 0.0);

        let stack = Stack::new(vec![Box::new(major), Box::new(et)]);

        // Degree 0 → semitone 0 → C4 ≈ 261.63 Hz
        let freq_c4 = stack.step(0.0);
        assert!(
            (freq_c4 - 261.63).abs() < EPSILON,
            "C4: got {}",
            freq_c4
        );

        // Degree 7 → semitone 12 → C5 ≈ 523.25 Hz
        let freq_c5 = stack.step(7.0);
        assert!(
            (freq_c5 - 523.25).abs() < EPSILON,
            "C5: got {}",
            freq_c5
        );

        // Octave relationship: C5 should be exactly double C4
        assert!(
            (freq_c5 / freq_c4 - 2.0).abs() < 0.001,
            "octave ratio: got {}",
            freq_c5 / freq_c4
        );
    }
}
