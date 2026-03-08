use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tracks the last time there was user activity in the system.
/// Used by the gatekeeper to determine when it's safe to promote cron jobs.
#[derive(Clone)]
pub struct ActivityTracker {
    last_activity: Arc<RwLock<DateTime<Utc>>>,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            last_activity: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Record that activity just occurred
    pub async fn record_activity(&self) {
        let mut last = self.last_activity.write().await;
        *last = Utc::now();
    }

    /// Get how long the system has been idle
    pub async fn idle_duration(&self) -> Duration {
        let last = *self.last_activity.read().await;
        Utc::now() - last
    }

    /// Check if the system has been idle for at least the given duration
    pub async fn is_idle_for(&self, duration: Duration) -> bool {
        self.idle_duration().await >= duration
    }
}

impl Default for ActivityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_activity_tracker() {
        let tracker = ActivityTracker::new();

        // Initial idle duration should be very small
        let initial = tracker.idle_duration().await;
        assert!(initial < Duration::seconds(1));

        // After waiting, idle duration should increase
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let after = tracker.idle_duration().await;
        assert!(after >= Duration::milliseconds(100));

        // After recording activity, idle duration should reset
        tracker.record_activity().await;
        let after_activity = tracker.idle_duration().await;
        assert!(after_activity < Duration::seconds(1));
    }

    #[tokio::test]
    async fn test_is_idle_for() {
        let tracker = ActivityTracker::new();

        // Should not be idle for 1 second immediately
        assert!(!tracker.is_idle_for(Duration::seconds(1)).await);

        // But should be idle for 0 milliseconds
        assert!(tracker.is_idle_for(Duration::milliseconds(0)).await);

        // Wait and check again
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        assert!(tracker.is_idle_for(Duration::milliseconds(100)).await);
    }
}
