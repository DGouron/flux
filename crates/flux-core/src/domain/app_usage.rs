use super::SessionId;

#[derive(Debug, Clone, PartialEq)]
pub struct AppUsage {
    pub session_id: SessionId,
    pub application_name: String,
    pub duration_seconds: i64,
}

impl AppUsage {
    pub fn new(session_id: SessionId, application_name: String) -> Self {
        Self {
            session_id,
            application_name,
            duration_seconds: 0,
        }
    }

    pub fn with_duration(session_id: SessionId, application_name: String, seconds: i64) -> Self {
        Self {
            session_id,
            application_name,
            duration_seconds: seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_usage_with_zero_duration() {
        let usage = AppUsage::new(1, "cursor".to_string());

        assert_eq!(usage.session_id, 1);
        assert_eq!(usage.application_name, "cursor");
        assert_eq!(usage.duration_seconds, 0);
    }

    #[test]
    fn with_duration_creates_usage_with_specified_seconds() {
        let usage = AppUsage::with_duration(42, "firefox".to_string(), 300);

        assert_eq!(usage.session_id, 42);
        assert_eq!(usage.application_name, "firefox");
        assert_eq!(usage.duration_seconds, 300);
    }
}
