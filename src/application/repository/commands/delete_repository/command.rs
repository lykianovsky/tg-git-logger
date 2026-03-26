use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct DeleteRepositoryCommand {
    pub id: RepositoryId,
}
