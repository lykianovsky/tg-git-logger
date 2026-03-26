use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct UpdateRepositoryCommand {
    pub id: RepositoryId,
    pub external_id: i64,
    pub name: String,
    pub owner: String,
    pub url: String,
}
