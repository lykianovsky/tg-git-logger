use crate::domain::user::value_objects::user_id::UserId;

/// Заведение карточки по упавшему тесту: ответственного и тег человек выбирает в боте
pub struct CreateTestFailureCardCommand {
    pub test_failure_id: i32,
    pub responsible_id: Option<u64>,
    pub tag: Option<String>,
    pub created_by_user_id: Option<UserId>,
}
