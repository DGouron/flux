use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct WeekStats {
    pub total_seconds: i64,
    pub session_count: usize,
    pub by_mode: HashMap<String, i64>,
    pub focus_applications: HashMap<String, i64>,
    pub distraction_applications: HashMap<String, i64>,
    pub total_distraction_seconds: i64,
    pub average_focus_score: u8,
}

#[derive(Debug, Clone)]
pub struct DigestStats {
    pub current_week: WeekStats,
    pub previous_week: Option<WeekStats>,
}

impl DigestStats {
    pub fn new(current_week: WeekStats, previous_week: Option<WeekStats>) -> Self {
        Self {
            current_week,
            previous_week,
        }
    }

    pub fn time_delta(&self) -> Option<i64> {
        self.previous_week
            .as_ref()
            .map(|prev| self.current_week.total_seconds - prev.total_seconds)
    }

    pub fn distraction_delta(&self) -> Option<i64> {
        self.previous_week.as_ref().map(|prev| {
            self.current_week.total_distraction_seconds - prev.total_distraction_seconds
        })
    }

    pub fn score_delta(&self) -> Option<i8> {
        self.previous_week.as_ref().map(|prev| {
            self.current_week.average_focus_score as i8 - prev.average_focus_score as i8
        })
    }

    pub fn session_delta(&self) -> Option<i64> {
        self.previous_week
            .as_ref()
            .map(|prev| self.current_week.session_count as i64 - prev.session_count as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_delta_returns_difference() {
        let current = WeekStats {
            total_seconds: 3600,
            ..Default::default()
        };
        let previous = WeekStats {
            total_seconds: 1800,
            ..Default::default()
        };

        let digest = DigestStats::new(current, Some(previous));

        assert_eq!(digest.time_delta(), Some(1800));
    }

    #[test]
    fn time_delta_returns_none_without_previous() {
        let current = WeekStats {
            total_seconds: 3600,
            ..Default::default()
        };

        let digest = DigestStats::new(current, None);

        assert_eq!(digest.time_delta(), None);
    }

    #[test]
    fn score_delta_handles_negative() {
        let current = WeekStats {
            average_focus_score: 70,
            ..Default::default()
        };
        let previous = WeekStats {
            average_focus_score: 85,
            ..Default::default()
        };

        let digest = DigestStats::new(current, Some(previous));

        assert_eq!(digest.score_delta(), Some(-15));
    }

    #[test]
    fn distraction_delta_computes_correctly() {
        let current = WeekStats {
            total_distraction_seconds: 300,
            ..Default::default()
        };
        let previous = WeekStats {
            total_distraction_seconds: 600,
            ..Default::default()
        };

        let digest = DigestStats::new(current, Some(previous));

        assert_eq!(digest.distraction_delta(), Some(-300));
    }
}
