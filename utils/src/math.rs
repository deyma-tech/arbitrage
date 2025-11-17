//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Percent

pub trait Percent {
    fn percent(&self, percent: f64) -> Self;
}

impl Percent for i32 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as i32
    }
}

impl Percent for i64 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as i64
    }
}

impl Percent for i128 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as i128
    }
}

impl Percent for u32 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as u32
    }
}

impl Percent for u64 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as u64
    }
}

impl Percent for u128 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as u128
    }
}

impl Percent for f32 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self as f64 * percent) / 100.0;
        result.round() as f32
    }
}

impl Percent for f64 {
    fn percent(&self, percent: f64) -> Self {
        let result = (*self * percent) / 100.0;
        result.round()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_i32() {
        assert_eq!(100_i32.percent(-10.0), -10);
        assert_eq!(100_i32.percent(20.0), 20);
        assert_eq!(100_i32.percent(-50.0), -50);
        assert_eq!(100_i32.percent(75.0), 75);
        assert_eq!(100_i32.percent(-90.0), -90);
        assert_eq!(100_i32.percent(99.0), 99);
        assert_eq!(100_i32.percent(100.0), 100);
    }

    #[test]
    fn test_percent_i64() {
        assert_eq!(100_i64.percent(-10.0), -10);
        assert_eq!(100_i64.percent(20.0), 20);
        assert_eq!(100_i64.percent(-50.0), -50);
        assert_eq!(100_i64.percent(75.0), 75);
        assert_eq!(100_i64.percent(-90.0), -90);
        assert_eq!(100_i64.percent(99.0), 99);
        assert_eq!(100_i64.percent(100.0), 100);
    }

    #[test]
    fn test_percent_i128() {
        assert_eq!(100_i128.percent(-10.0), -10);
        assert_eq!(100_i128.percent(20.0), 20);
        assert_eq!(100_i128.percent(-50.0), -50);
        assert_eq!(100_i128.percent(75.0), 75);
        assert_eq!(100_i128.percent(-90.0), -90);
        assert_eq!(100_i128.percent(99.0), 99);
        assert_eq!(100_i128.percent(100.0), 100);
    }

    #[test]
    fn test_percent_u32() {
        assert_eq!(100_u32.percent(-10.0), 0);
        assert_eq!(100_u32.percent(20.0), 20);
        assert_eq!(100_u32.percent(-50.0), 0);
        assert_eq!(100_u32.percent(75.0), 75);
        assert_eq!(100_u32.percent(-90.0), 0);
        assert_eq!(100_u32.percent(99.0), 99);
        assert_eq!(100_u32.percent(100.0), 100);
    }

    #[test]
    fn test_percent_u64() {
        assert_eq!(100_u64.percent(-10.0), 0);
        assert_eq!(100_u64.percent(20.0), 20);
        assert_eq!(100_u64.percent(-50.0), 0);
        assert_eq!(100_u64.percent(75.0), 75);
        assert_eq!(100_u64.percent(-90.0), 0);
        assert_eq!(100_u64.percent(99.0), 99);
        assert_eq!(100_u64.percent(100.0), 100);
    }

    #[test]
    fn test_percent_u128() {
        assert_eq!(100_u128.percent(-10.0), 0);
        assert_eq!(100_u128.percent(20.0), 20);
        assert_eq!(100_u128.percent(-50.0), 0);
        assert_eq!(100_u128.percent(75.0), 75);
        assert_eq!(100_u128.percent(-90.0), 0);
        assert_eq!(100_u128.percent(99.0), 99);
        assert_eq!(100_u128.percent(100.0), 100);
    }

    #[test]
    fn test_percent_f32() {
        assert_eq!(100.0_f32.percent(-10.0), -10.0);
        assert_eq!(100.0_f32.percent(20.0), 20.0);
        assert_eq!(100.0_f32.percent(-50.0), -50.0);
        assert_eq!(100.0_f32.percent(75.0), 75.0);
        assert_eq!(100.0_f32.percent(-90.0), -90.0);
        assert_eq!(100.0_f32.percent(99.0), 99.0);
        assert_eq!(100.0_f32.percent(100.0), 100.0);
    }

    #[test]
    fn test_percent_f64() {
        assert_eq!(100.0_f64.percent(-10.0), -10.0);
        assert_eq!(100.0_f64.percent(20.0), 20.0);
        assert_eq!(100.0_f64.percent(-50.0), -50.0);
        assert_eq!(100.0_f64.percent(75.0), 75.0);
        assert_eq!(100.0_f64.percent(-90.0), -90.0);
        assert_eq!(100.0_f64.percent(99.0), 99.0);
        assert_eq!(100.0_f64.percent(100.0), 100.0);
    }
}
