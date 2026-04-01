use crate::domain::user::value_objects::user_id::UserId;

pub struct ToggleUserActiveCommand {
    pub user_id: UserId,
    pub is_active: bool,
}
