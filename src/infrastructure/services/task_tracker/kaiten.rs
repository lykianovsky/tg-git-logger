use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::task::value_objects::task_id::TaskId;
use regex::Regex;

pub struct KaitenTaskTrackerService {
    extract_pattern: String,
}

impl KaitenTaskTrackerService {
    pub fn new(extract_pattern: String) -> Self {
        Self { extract_pattern }
    }
}

impl TaskTrackerService for KaitenTaskTrackerService {
    fn extract_task_id_by_pattern(&self, text: &str) -> Option<TaskId> {
        let regex = Regex::new(self.extract_pattern.as_str()).ok()?;

        regex
            .captures(text)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .map(TaskId)
    }

    fn extract_task_match(&self, text: &str) -> Option<(String, TaskId)> {
        let regex = Regex::new(self.extract_pattern.as_str()).ok()?;

        regex.captures(text).and_then(|caps| {
            let full_match = caps.get(0)?.as_str().to_string();
            let task_id = caps.get(1)?.as_str().parse::<u64>().ok().map(TaskId)?;
            Some((full_match, task_id))
        })
    }

    fn extract_match_with_pattern(&self, text: &str, pattern: &str) -> Option<(String, TaskId)> {
        let regex = Regex::new(pattern).ok()?;
        regex.captures(text).and_then(|caps| {
            let full_match = caps.get(0)?.as_str().to_string();
            let task_id = caps.get(1)?.as_str().parse::<u64>().ok().map(TaskId)?;
            Some((full_match, task_id))
        })
    }

    fn extract_all_matches_with_pattern(&self, text: &str, pattern: &str) -> Vec<(String, TaskId)> {
        let Ok(regex) = Regex::new(pattern) else {
            return vec![];
        };

        regex
            .captures_iter(text)
            .filter_map(|caps| {
                let full_match = caps.get(0)?.as_str().to_string();
                let task_id = caps.get(1)?.as_str().parse::<u64>().ok().map(TaskId)?;
                Some((full_match, task_id))
            })
            .collect()
    }
}
