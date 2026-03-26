use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct UpdateRepositoryTaskTrackerCommand {
    pub repository_id: RepositoryId,
    pub space_id: i32,
    pub qa_column_id: i32,
    pub extract_pattern_regexp: String,
    pub path_to_card: String,
}
