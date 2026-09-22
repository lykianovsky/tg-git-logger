use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct ListTestBlocksQuery {
    pub repository_id: RepositoryId,
}
