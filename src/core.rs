// Core ScaleVec type — a numeric vector with period-wrapping step() function.
// Ported from ScaleVec::Scale::Intervalic in p6-ScaleVec.

use super::Scale;

/// A flexible music representation as a vector of numeric values.
///
/// ScaleVec can represent scales, chords, rhythms, or any ordered numeric
/// structure. The `step()` function maps abstract indices to concrete values
/// with automatic period-wrapping (octave folding) and fractional interpolation.
#[derive(Debug, Clone)]
pub struct ScaleVec {
    /// The pitch (or value) vector — ordered numeric values.
    vector: Vec<f64>,
    /// Differences between consecutive elements of the vector.
    interval_vector: Vec<f64>,
    /// The period at which the vector repeats (e.g. 12 for chromatic semitones).
    repeat_interval: f64,
}

impl ScaleVec {
    /// Create a new ScaleVec from a vector of values.
    ///
    /// The `repeat_interval` is derived from `last - first` by default.
    /// Pass `None` for automatic derivation, or `Some(value)` to override.
    pub fn new(vector: Vec<f64>, repeat_interval: Option<f64>) -> Self {
        assert!(
            vector.len() >= 2,
            "ScaleVec requires at least 2 elements (root + one interval)"
        );

        let ri = repeat_interval.unwrap_or_else(|| {
            vector.last().unwrap() - vector.first().unwrap()
        });

        // Derive the interval vector (differences between consecutive elements)
        let interval_vector = pv_to_iv(&vector);

        ScaleVec {
            vector,
            interval_vector,
            repeat_interval: ri,
        }
    }

    /// The root (first element) of the vector.
    pub fn root(&self) -> f64 {
        self.vector[0]
    }

    /// The period at which the pattern repeats.
    pub fn repeat_interval(&self) -> f64 {
        self.repeat_interval
    }

    /// Access the raw pitch vector.
    pub fn vector(&self) -> &[f64] {
        &self.vector
    }

    /// Access the interval vector (differences between consecutive elements).
    pub fn interval_vector(&self) -> &[f64] {
        &self.interval_vector
    }

    /// The number of steps in one period (i.e. number of scale degrees).
    /// This is `vector.len()` — the last element is the period boundary,
    /// so the number of unique scale degrees is `len - 1`, but we include
    /// the boundary for interpolation purposes.
    fn scale_pv_len(&self) -> usize {
        self.vector.len()
    }

    /// Convert a step to its pitch class (position within one period).
    /// Operates purely on step indices — root value is not involved.
    fn step_to_pc(&self, step: f64) -> f64 {
        let period_len = (self.scale_pv_len() - 1) as f64;
        step.rem_euclid(period_len)
    }

    /// Convert a step to its octave (which period repetition it's in).
    fn step_to_octave(&self, step: f64) -> f64 {
        let pc = self.step_to_pc(step);
        let period_len = (self.scale_pv_len() - 1) as f64;
        (step - pc) / period_len
    }

    /// Convert a value to its pitch class within the repeat interval.
    fn value_to_pc(&self, value: f64) -> f64 {
        (value - self.root()).rem_euclid(self.repeat_interval)
    }

    /// Convert a value to its octave number.
    fn value_to_octave(&self, value: f64) -> f64 {
        let pc = self.value_to_pc(value);
        ((value - pc) / self.repeat_interval) - (self.root() / self.repeat_interval)
    }

    /// Find the whole step index for a given value within the scale.
    fn value_to_whole_step(&self, value: f64) -> usize {
        let pc = self.value_to_pc(value);
        let scale_pv: Vec<f64> = self.scale_pv();
        for (i, val) in scale_pv.iter().enumerate() {
            if i + 1 < scale_pv.len() {
                let current_pc = self.value_to_pc(*val);
                let next_pc = self.value_to_pc(scale_pv[i + 1]);
                if current_pc <= pc && pc < next_pc {
                    return i;
                }
            }
        }
        scale_pv.len() - 2
    }

    /// Calculate the fractional sub-step for a value.
    fn value_to_sub_step(&self, value: f64) -> f64 {
        let whole_step = self.value_to_whole_step(value);
        let step_val = self.step_internal(whole_step as f64);
        let next_val = self.step_internal((whole_step + 1) as f64);
        let interval = next_val - step_val;
        if interval.abs() < f64::EPSILON {
            return 0.0;
        }
        (self.value_to_pc(value) - self.value_to_pc(step_val)) / interval
    }

    /// The full scale pitch vector including the period boundary.
    fn scale_pv(&self) -> Vec<f64> {
        let mut pv: Vec<f64> = self.vector.clone();
        // Ensure the period boundary is present
        let boundary = self.root() + self.repeat_interval;
        if let Some(last) = pv.last() {
            if (*last - boundary).abs() > f64::EPSILON {
                pv.push(boundary);
            }
        }
        pv
    }

    /// The scale interval vector.
    fn scale_iv(&self) -> Vec<f64> {
        pv_to_iv(&self.scale_pv())
    }

    /// Internal step function (matches Scale::Intervalic.step from Raku).
    fn step_internal(&self, step: f64) -> f64 {
        let pc = self.step_to_pc(step);
        let pc_floor = pc.floor() as usize;
        let scale_pv = self.scale_pv();
        let scale_iv = self.scale_iv();

        let pc_floor_clamped = pc_floor.min(scale_pv.len() - 1);
        let iv_index = pc_floor_clamped.min(scale_iv.len() - 1);

        self.root()
            + (self.step_to_octave(step) * self.repeat_interval)  // octave offset
            + self.value_to_pc(scale_pv[pc_floor_clamped])        // whole step value
            + ((pc - pc_floor as f64) * scale_iv[iv_index])       // fractional interpolation
    }

    /// Inverse of step — given a value, recover the step index.
    ///
    /// This is the reflexive mapping: if `step(n) = v`, then
    /// `reflexive_step(v) ≈ n`.
    pub fn reflexive_step(&self, value: f64) -> f64 {
        let period_len = (self.scale_pv_len() - 1) as f64;
        let octave = self.value_to_octave(value);
        let whole = self.value_to_whole_step(value);
        let sub = self.value_to_sub_step(value);

        (octave * period_len) + whole as f64 + sub
    }

    // --- Transforms ---

    /// Shift all values by a constant interval.
    pub fn transpose(&self, interval: f64) -> ScaleVec {
        let vector = self.vector.iter().map(|v| v + interval).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Negate all values (mirror inversion around zero).
    pub fn mirror_inversion(&self) -> ScaleVec {
        let vector = self.vector.iter().map(|v| -v).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Reverse the vector order.
    pub fn retrograde(&self) -> ScaleVec {
        let mut vector = self.vector.clone();
        vector.reverse();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Expand intervals by adding a constant to each value.
    pub fn augment_steps(&self, steps: f64) -> ScaleVec {
        let vector = self.vector.iter().map(|v| v + steps).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Expand intervals by a ratio.
    pub fn augment_ratio(&self, ratio: f64) -> ScaleVec {
        let vector = self.vector.iter().map(|v| v + (v * ratio)).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Contract intervals by subtracting a constant from each value.
    pub fn diminish_steps(&self, steps: f64) -> ScaleVec {
        let vector = self.vector.iter().map(|v| v - steps).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Contract intervals by a ratio.
    pub fn diminish_ratio(&self, ratio: f64) -> ScaleVec {
        let vector = self.vector.iter().map(|v| v - (v * ratio)).collect();
        ScaleVec::new(vector, Some(self.repeat_interval))
    }

    /// Project this vector through another scale's step() function.
    pub fn map_onto(&self, scale: &dyn Scale) -> ScaleVec {
        let vector = self.scale_pv().iter().map(|v| scale.step(*v)).collect();
        ScaleVec::new(vector, None)
    }

    /// Euclidean distance to another ScaleVec.
    pub fn distance(&self, other: &ScaleVec) -> f64 {
        self.vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| (b - a).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

impl Scale for ScaleVec {
    fn step(&self, n: f64) -> f64 {
        self.step_internal(n)
    }
}

/// Convert a pitch vector to an interval vector (consecutive differences).
fn pv_to_iv(vector: &[f64]) -> Vec<f64> {
    vector
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

/// Convert an interval vector back to a pitch vector.
#[allow(dead_code)]
fn iv_to_pv(intervals: &[f64], root: f64) -> Vec<f64> {
    let mut pv = Vec::with_capacity(intervals.len() + 1);
    pv.push(root);
    let mut prev = root;
    for &iv in intervals {
        prev += iv;
        pv.push(prev);
    }
    pv
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tolerance for floating-point comparisons (~0.01 cents is inaudible).
    const EPSILON: f64 = 0.001;

    fn assert_approx_eq(a: f64, b: f64, msg: &str) {
        assert!(
            (a - b).abs() < EPSILON,
            "{}: expected {}, got {} (diff {})",
            msg,
            b,
            a,
            (a - b).abs()
        );
    }

    fn major_scale() -> ScaleVec {
        // C major in semitones: C D E F G A B C
        ScaleVec::new(vec![0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0], None)
    }

    fn chromatic_scale() -> ScaleVec {
        ScaleVec::new(
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
            None,
        )
    }

    #[test]
    fn test_basic_properties() {
        let scale = major_scale();
        assert_eq!(scale.root(), 0.0);
        assert_eq!(scale.repeat_interval(), 12.0);
        assert_eq!(scale.vector().len(), 8);
        assert_eq!(scale.interval_vector().len(), 7);
    }

    #[test]
    fn test_interval_vector() {
        let scale = major_scale();
        let iv = scale.interval_vector();
        // Major scale intervals: W W H W W W H (2 2 1 2 2 2 1)
        assert_approx_eq(iv[0], 2.0, "interval 0");
        assert_approx_eq(iv[1], 2.0, "interval 1");
        assert_approx_eq(iv[2], 1.0, "interval 2");
        assert_approx_eq(iv[3], 2.0, "interval 3");
        assert_approx_eq(iv[4], 2.0, "interval 4");
        assert_approx_eq(iv[5], 2.0, "interval 5");
        assert_approx_eq(iv[6], 1.0, "interval 6");
    }

    #[test]
    fn test_step_integer_within_period() {
        let scale = major_scale();
        // Step 0 = root = 0
        assert_approx_eq(scale.step(0.0), 0.0, "step 0");
        // Step 1 = D = 2
        assert_approx_eq(scale.step(1.0), 2.0, "step 1");
        // Step 2 = E = 4
        assert_approx_eq(scale.step(2.0), 4.0, "step 2");
        // Step 3 = F = 5
        assert_approx_eq(scale.step(3.0), 5.0, "step 3");
        // Step 4 = G = 7
        assert_approx_eq(scale.step(4.0), 7.0, "step 4");
    }

    #[test]
    fn test_step_octave_wrapping() {
        let scale = major_scale();
        // Step 7 should wrap to the next octave's root = 12
        assert_approx_eq(scale.step(7.0), 12.0, "step 7 (octave)");
        // Step 8 = next octave's D = 14
        assert_approx_eq(scale.step(8.0), 14.0, "step 8");
        // Step 14 = two octaves up = 24
        assert_approx_eq(scale.step(14.0), 24.0, "step 14 (two octaves)");
    }

    #[test]
    fn test_step_negative() {
        let scale = major_scale();
        // Step -1 should be one step below root = B below = -1
        assert_approx_eq(scale.step(-1.0), -1.0, "step -1");
        // Step -7 = one octave below root = -12
        assert_approx_eq(scale.step(-7.0), -12.0, "step -7");
    }

    #[test]
    fn test_step_fractional() {
        let scale = major_scale();
        // Step 0.5 should interpolate between 0 and 2 = 1.0
        assert_approx_eq(scale.step(0.5), 1.0, "step 0.5");
        // Step 2.5 should interpolate between 4 and 5 = 4.5
        assert_approx_eq(scale.step(2.5), 4.5, "step 2.5");
    }

    #[test]
    fn test_chromatic_identity() {
        let scale = chromatic_scale();
        // For a chromatic scale, step(n) should equal n for integer n in [0, 12]
        for i in 0..=12 {
            assert_approx_eq(
                scale.step(i as f64),
                i as f64,
                &format!("chromatic step {}", i),
            );
        }
    }

    #[test]
    fn test_reflexive_step_round_trip() {
        let scale = major_scale();
        // For integer steps, reflexive_step(step(n)) should ≈ n
        for i in 0..7 {
            let value = scale.step(i as f64);
            let recovered = scale.reflexive_step(value);
            assert_approx_eq(
                recovered,
                i as f64,
                &format!("round-trip step {}", i),
            );
        }
    }

    #[test]
    fn test_transpose() {
        let scale = major_scale();
        let transposed = scale.transpose(5.0);
        // Root should be 5.0 now
        assert_approx_eq(transposed.root(), 5.0, "transposed root");
        // Step 0 should be 5.0
        assert_approx_eq(transposed.step(0.0), 5.0, "transposed step 0");
    }

    #[test]
    fn test_pv_to_iv_and_back() {
        let pv = vec![0.0, 2.0, 4.0, 5.0, 7.0];
        let iv = pv_to_iv(&pv);
        let reconstructed = iv_to_pv(&iv, 0.0);
        for (a, b) in pv.iter().zip(reconstructed.iter()) {
            assert_approx_eq(*a, *b, "pv round-trip");
        }
    }
}
