use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq)]
pub enum ColorCode {
    None,
    Low,
    Medium,
    High,
}

pub fn get_size_color_code(size: u64) -> ColorCode {
    if size < 1000 * 1000 * 90 {
        ColorCode::Low
    } else if size < 1000 * 1000 * 900 {
        ColorCode::Medium
    } else {
        ColorCode::High
    }
}

pub fn get_time_color_code(now: &SystemTime, time: &Option<SystemTime>) -> ColorCode {
    match time {
        None => ColorCode::None, // Wouldn't be displayed anyway
        Some(system_time) => match now.duration_since(*system_time) {
            Err(_) => ColorCode::None, // Future time; shouldn't happen
            Ok(duration) => {
                if duration < Duration::from_days(14) {
                    ColorCode::Low
                } else if duration < Duration::from_days(60) {
                    ColorCode::Medium
                } else {
                    ColorCode::High
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_color_coding() {
        assert_eq!(get_size_color_code(1000), ColorCode::Low);
        assert_eq!(get_size_color_code(1000 * 1000), ColorCode::Low);
        assert_eq!(get_size_color_code(50 * 1000 * 1000), ColorCode::Low);
        assert_eq!(get_size_color_code(80 * 1000 * 1000), ColorCode::Low);
        assert_eq!(get_size_color_code(90 * 1000 * 1000), ColorCode::Medium);
        assert_eq!(get_size_color_code(100 * 1000 * 1000), ColorCode::Medium);
        assert_eq!(get_size_color_code(500 * 1000 * 1000), ColorCode::Medium);
        assert_eq!(get_size_color_code(800 * 1000 * 1000), ColorCode::Medium);
        assert_eq!(get_size_color_code(900 * 1000 * 1000), ColorCode::High);
        assert_eq!(get_size_color_code(1000 * 1000 * 1000), ColorCode::High);
    }

    #[test]
    fn test_time_color_coding() {
        let now = SystemTime::now();

        assert_eq!(get_time_color_code(&now, &None), ColorCode::None);
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(1))),
            ColorCode::Low
        );
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(10))),
            ColorCode::Low
        );
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(20))),
            ColorCode::Medium
        );
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(50))),
            ColorCode::Medium
        );
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(60))),
            ColorCode::High
        );
        assert_eq!(
            get_time_color_code(&now, &Some(now - Duration::from_days(70))),
            ColorCode::High
        );
    }
}
