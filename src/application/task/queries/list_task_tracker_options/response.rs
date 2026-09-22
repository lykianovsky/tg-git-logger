/// Вариант выбора в форме: идентификатор уходит в карточку, название — на кнопку
pub struct TaskTrackerOption {
    pub id: u64,
    pub name: String,
}

pub struct ListTaskTrackerOptionsResponse {
    pub options: Vec<TaskTrackerOption>,
}
