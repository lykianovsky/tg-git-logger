use crate::domain::task::value_objects::task_id::TaskId;

pub struct GetTaskCardQuery {
    pub task_id: TaskId,
}
