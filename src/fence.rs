// Fence: clamps/wraps values within bounds with period wrapping.
// Ported from ScaleVec::Scale::Fence in p6-ScaleVec.

use super::Scale;

/// Constrains values within a bounded range using periodic wrapping.
///
/// Useful for MIDI note range (0–127), pitch class space (0–12), etc.
/// Values outside the bounds are wrapped back into range using the
/// repeat interval.
#[derive(Debug, Clone)]
pub struct Fence {
    pub upper_limit: f64,
    pub lower_limit: f64,
    pub repeat_interval: f64,
}

impl Fence {
    pub fn new(lower_limit: f64, upper_limit: f64, repeat_interval: f64) -> Self {
        Fence {
            upper_limit,
            lower_limit,
            repeat_interval,
        }
    }
}

impl Scale for Fence {
    fn step(&self, value: f64) -> f64 {
        if value >= self.upper_limit {
            // Wrap down from upper limit
            self.upper_limit
                - (self.repeat_interval
                    - ((value - self.upper_limit) % self.repeat_interval))
        } else if value < self.lower_limit {
            // Wrap up from lower limit
            self.lower_limit + (value.rem_euclid(self.repeat_interval))
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_within_bounds() {
        let fence = Fence::new(0.0, 127.0, 12.0);
        assert_eq!(fence.step(60.0), 60.0);
        assert_eq!(fence.step(0.0), 0.0);
        assert_eq!(fence.step(126.0), 126.0);
    }

    #[test]
    fn test_above_upper_limit() {
        let fence = Fence::new(0.0, 128.0, 12.0);
        // 128 is at the limit, should wrap
        let result = fence.step(130.0);
        assert!(result >= 0.0 && result < 128.0, "got {}", result);
    }

    #[test]
    fn test_below_lower_limit() {
        let fence = Fence::new(0.0, 127.0, 12.0);
        let result = fence.step(-1.0);
        assert!(result >= 0.0 && result < 127.0, "got {}", result);
    }

    #[test]
    fn test_pitch_class_space() {
        // Pitch class space: 0–12, period 12
        let pc = Fence::new(0.0, 12.0, 12.0);
        assert_eq!(pc.step(5.0), 5.0);
    }
}
