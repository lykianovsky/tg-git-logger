use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct CreateRepositoryTaskTrackerCommand {
    pub repository_id: RepositoryId,
    pub space_id: i32,
    /// Доска, на которой выбраны колонки
    pub board_id: Option<i32>,
    /// Колонка новых задач — в неё уходят карточки по упавшим тестам
    pub new_task_column_id: Option<i32>,
    pub qa_column_id: i32,
    pub review_column_id: i32,
    pub extract_pattern_regexp: String,
    pub path_to_card: String,
}
