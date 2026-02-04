use crate::domain::user::repository::CreateUserException;
use crate::domain::user::{entities::User, repository::UserRepository};
use std::sync::Arc;

pub struct CreateUserUseCase {
    pub repository: Arc<dyn UserRepository + Send + Sync>,
}

impl CreateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user: User) -> Result<User, CreateUserException> {
        self.repository.create(&user).await?;

        Ok(user)
    }
}
