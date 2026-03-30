use crate::domain::task::value_objects::task_id::TaskId;

pub struct MoveTaskToTestExecutorCommand {
    pub task_id: TaskId,
    pub column_id: u64,
}
