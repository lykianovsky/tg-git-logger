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

        regex.captures(text).and_then(|caps| caps.get(1)).map(|m| {
            // TODO
            let id = m.as_str().parse::<u64>().unwrap();
            TaskId(id)
        })
    }
}
