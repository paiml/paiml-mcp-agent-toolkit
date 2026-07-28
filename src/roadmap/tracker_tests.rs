#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::roadmap::Priority;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_velocity_tracker_creation() {
        let tracker = VelocityTracker::new("sprint-45");

        assert_eq!(tracker.sprint_id, "sprint-45");
        assert!(tracker.tasks_completed.is_empty());
        assert!(tracker.quality_scores.is_empty());
        assert_eq!(tracker.average_cycle_time, Duration::new(0, 0));
        assert!(tracker.burndown_data.is_empty());
    }

    #[test]
    fn test_add_completed_task() {
        let mut tracker = VelocityTracker::new("test-sprint");
        let started = Utc.with_ymd_and_hms(2025, 9, 1, 10, 0, 0).unwrap();
        let completed = Utc.with_ymd_and_hms(2025, 9, 2, 15, 0, 0).unwrap();

        let task = CompletedTask {
            task_id: "TASK-001".to_string(),
            started_at: started,
            completed_at: completed,
            complexity: Complexity::Medium,
            quality_score: 85.0,
            rework_count: 1,
        };

        tracker.add_completed_task(task.clone());

        assert_eq!(tracker.tasks_completed.len(), 1);
        assert_eq!(tracker.tasks_completed[0].task_id, "TASK-001");
        assert_eq!(tracker.tasks_completed[0].quality_score, 85.0);
    }

    #[test]
    fn test_calculate_velocity() {
        let mut tracker = VelocityTracker::new("test-sprint");

        // Set start time to 2 days ago to ensure velocity calculation works
        tracker.started_at = Utc::now() - chrono::Duration::days(2);

        // Add some completed tasks
        let task1 = CompletedTask {
            task_id: "TASK-001".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            complexity: Complexity::High,
            quality_score: 90.0,
            rework_count: 0,
        };

        let task2 = CompletedTask {
            task_id: "TASK-002".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            complexity: Complexity::Medium,
            quality_score: 85.0,
            rework_count: 1,
        };

        tracker.add_completed_task(task1);
        tracker.add_completed_task(task2);

        let velocity = tracker.calculate_velocity();
        assert!(velocity > 0.0);
    }

    #[test]
    fn test_add_quality_score() {
        let mut tracker = VelocityTracker::new("test-sprint");

        tracker.add_quality_score("TASK-001", 92.5);

        assert_eq!(tracker.quality_scores.len(), 1);
        assert_eq!(tracker.quality_scores[0].task_id, "TASK-001");
        assert_eq!(tracker.quality_scores[0].score, 92.5);
    }

    #[test]
    fn test_update_burndown() {
        let mut tracker = VelocityTracker::new("test-sprint");

        tracker.update_burndown(5, 10);
        tracker.update_burndown(6, 8);

        assert_eq!(tracker.burndown_data.len(), 2);
        assert_eq!(tracker.burndown_data[0].day, 5);
        assert_eq!(tracker.burndown_data[0].remaining_tasks, 10);
        assert_eq!(tracker.burndown_data[1].day, 6);
        assert_eq!(tracker.burndown_data[1].remaining_tasks, 8);
    }

    #[test]
    fn test_get_average_quality() {
        let mut tracker = VelocityTracker::new("test-sprint");

        tracker.add_quality_score("TASK-001", 90.0);
        tracker.add_quality_score("TASK-002", 85.0);
        tracker.add_quality_score("TASK-003", 95.0);

        let avg_quality = tracker.get_average_quality();
        assert_eq!(avg_quality, 90.0); // (90+85+95)/3 = 90
    }

    #[test]
    fn test_get_cycle_time_stats() {
        let mut tracker = VelocityTracker::new("test-sprint");

        let base_time = Utc::now();
        let task = CompletedTask {
            task_id: "TASK-001".to_string(),
            started_at: base_time,
            completed_at: base_time + chrono::Duration::hours(24), // 1 day
            complexity: Complexity::Medium,
            quality_score: 85.0,
            rework_count: 0,
        };

        tracker.add_completed_task(task);
        let stats = tracker.get_cycle_time_stats();

        assert!(stats.min_cycle_time > Duration::new(0, 0));
        assert_eq!(stats.task_count, 1);
    }

    #[test]
    fn test_generate_report() {
        let sprint = Sprint {
            version: "v2.43.0".to_string(),
            title: "Test Sprint".to_string(),
            start_date: Utc::now(),
            end_date: Utc::now() + chrono::Duration::days(7),
            priority: Priority::P1,
            tasks: vec![],
            definition_of_done: vec!["All tests pass".to_string()],
            quality_gates: vec!["Code coverage > 80%".to_string()],
        };

        let report = ProgressReporter::generate_report(&sprint).unwrap();

        assert!(report.contains("v2.43.0"));
        assert!(report.contains("Test Sprint"));
        assert!(report.contains("All tests pass"));
        assert!(report.contains("Code coverage > 80%"));
    }

    #[test]
    fn test_empty_quality_scores_average() {
        let tracker = VelocityTracker::new("empty-sprint");
        assert_eq!(tracker.get_average_quality(), 0.0);
    }
}

