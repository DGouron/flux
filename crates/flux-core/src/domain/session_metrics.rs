use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub session_id: SessionId,
    pub context_switch_count: u32,
    pub total_short_bursts: u32,
    pub short_bursts_by_app: HashMap<String, u32>,
}

impl SessionMetrics {
    pub fn new(
        session_id: SessionId,
        context_switch_count: u32,
        short_bursts_by_app: HashMap<String, u32>,
    ) -> Self {
        let total_short_bursts = short_bursts_by_app.values().sum();

        Self {
            session_id,
            context_switch_count,
            total_short_bursts,
            short_bursts_by_app,
        }
    }

    pub fn focus_score(&self) -> u8 {
        if self.context_switch_count == 0 {
            return 100;
        }

        let penalty = (self.context_switch_count as f64 * 2.0
            + self.total_short_bursts as f64 * 5.0)
            .min(100.0);

        (100.0 - penalty).max(0.0) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_computes_total_short_bursts() {
        let mut short_bursts = HashMap::new();
        short_bursts.insert("discord".to_string(), 5);
        short_bursts.insert("slack".to_string(), 3);

        let metrics = SessionMetrics::new(1, 10, short_bursts);

        assert_eq!(metrics.total_short_bursts, 8);
    }

    #[test]
    fn focus_score_is_100_with_no_switches() {
        let metrics = SessionMetrics::new(1, 0, HashMap::new());
        assert_eq!(metrics.focus_score(), 100);
    }

    #[test]
    fn focus_score_decreases_with_switches() {
        let metrics = SessionMetrics::new(1, 10, HashMap::new());
        assert!(metrics.focus_score() < 100);
    }

    #[test]
    fn focus_score_penalizes_short_bursts_more() {
        let mut short_bursts = HashMap::new();
        short_bursts.insert("discord".to_string(), 10);

        let metrics_with_bursts = SessionMetrics::new(1, 5, short_bursts);
        let metrics_without_bursts = SessionMetrics::new(1, 5, HashMap::new());

        assert!(metrics_with_bursts.focus_score() < metrics_without_bursts.focus_score());
    }
}
