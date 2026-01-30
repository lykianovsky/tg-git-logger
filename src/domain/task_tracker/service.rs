use std::fmt;

#[derive(Debug)]
pub enum TaskTrackerMoveCardError {
    RequestFailed
}

impl fmt::Display for TaskTrackerMoveCardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskTrackerMoveCardError::RequestFailed => write!(f, "Request failed error"),
        }
    }
}

#[derive(Debug)]
pub struct TaskTrackerTaskId(pub String);

#[derive(Debug)]
pub struct TaskTrackerColumnId(pub String);

impl fmt::Display for TaskTrackerTaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for TaskTrackerColumnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[async_trait::async_trait]
pub trait TaskTrackerService: Send + Sync {
    async fn move_card(&self, task_id: &TaskTrackerTaskId, column_id: &TaskTrackerColumnId) -> Result<(), TaskTrackerMoveCardError>;

    fn linkify_tasks_in_text(&self, text: &str) -> String;

    fn create_task_link(&self, id: &TaskTrackerTaskId, text: &str) -> String;

    fn extract_task_id(&self, text: &str) -> Option<TaskTrackerTaskId>;
}