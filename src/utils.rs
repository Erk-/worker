use humantime::FormattedDuration;
use std::time::Duration;

/// Converts track time (in milliseconds) to a human-readable time.
pub fn track_length_readable(ms: u64) -> FormattedDuration {
    humantime::format_duration(Duration::from_secs(ms / 1000))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_track_length_readable() {
        assert_eq!(super::track_length_readable(1000).to_string(), "1s");
        assert_eq!(super::track_length_readable(62_043).to_string(), "1m 2s");
        assert_eq!(
            super::track_length_readable(3_674_000).to_string(),
            "1h 1m 14s",
        );
    }
}
