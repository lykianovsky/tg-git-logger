use crate::domain::repository::entities::repository::Repository;

pub struct GetAllRepositoriesResponse {
    pub repositories: Vec<Repository>,
}
