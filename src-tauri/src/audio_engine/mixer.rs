/// Simple stereo mixer bus: mix multiple f32 interleaved buffers.
pub fn apply_gain(buf: &mut [f32], gain: f32) {
    for s in buf.iter_mut() {
        *s *= gain;
    }
}

pub fn peak(buf: &[f32]) -> f32 {
    buf.iter().map(|sample| sample.abs()).fold(0.0_f32, f32::max)
}

/// Apply soft-knee limiter to prevent clipping.
pub fn limit(buf: &mut [f32], ceiling: f32) {
    for s in buf.iter_mut() {
        let abs = s.abs();
        if abs > ceiling {
            *s = s.signum() * ceiling;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{limit, peak};

    #[test]
    fn peak_uses_absolute_value() {
        let samples = [-0.9, 0.2, -0.4, 0.1];
        assert!((peak(&samples) - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn limit_clamps_values_to_ceiling() {
        let mut samples = [-1.4, -0.5, 0.25, 1.2];
        limit(&mut samples, 1.0);
        assert_eq!(samples, [-1.0, -0.5, 0.25, 1.0]);
    }
}
