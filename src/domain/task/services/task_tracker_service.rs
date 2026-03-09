use crate::domain::task::value_objects::task_id::TaskId;

pub trait TaskTrackerService: Send + Sync {
    fn extract_task_id_by_pattern(&self, text: &str) -> Option<TaskId>;
}
