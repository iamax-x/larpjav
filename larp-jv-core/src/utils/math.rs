//! Math utilities

/// Clamp value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { min } else if value > max { max } else { value }
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * clamp(t, 0.0, 1.0)
}

/// Smooth step interpolation
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Map value from one range to another
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}

/// Degrees to radians
pub fn deg2rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

/// Radians to degrees
pub fn rad2deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

/// Distance between two 2D points
pub fn distance2d(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

/// Distance between two 3D points
pub fn distance3d(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
}

/// Round to decimal places
pub fn round_to(value: f32, decimals: u32) -> f32 {
    let factor = 10f32.powi(decimals as i32);
    (value * factor).round() / factor
}

/// Check if approximately equal
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Next power of two
pub fn next_pow2(mut n: u32) -> u32 {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

/// Previous power of two
pub fn prev_pow2(n: u32) -> u32 {
    if n == 0 { return 0; }
    let mut n = n;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n - (n >> 1)
}

/// Signum for integers
pub fn signum_i32(n: i32) -> i32 {
    if n > 0 { 1 } else if n < 0 { -1 } else { 0 }
}

/// Signum for floats
pub fn signum_f32(n: f32) -> f32 {
    if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, -1.0), 0.0); // clamped
        assert_eq!(lerp(0.0, 10.0, 2.0), 10.0); // clamped
    }

    #[test]
    fn test_distance() {
        assert_eq!(distance2d(0.0, 0.0, 3.0, 4.0), 5.0);
        assert_eq!(distance3d(0.0, 0.0, 0.0, 1.0, 2.0, 2.0), 3.0);
    }

    #[test]
    fn test_pow2() {
        assert_eq!(next_pow2(5), 8);
        assert_eq!(next_pow2(16), 16);
        assert_eq!(prev_pow2(5), 4);
        assert_eq!(prev_pow2(16), 16);
    }
}
