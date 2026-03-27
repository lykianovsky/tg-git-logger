use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct UpdateRepositoryCommand {
    pub id: RepositoryId,
    pub name: String,
    pub owner: String,
    pub url: String,
}
