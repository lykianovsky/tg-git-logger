use crate::domain::repository::entities::repository::Repository;

pub struct GetUserBoundRepositoriesResponse {
    pub repositories: Vec<Repository>,
}
