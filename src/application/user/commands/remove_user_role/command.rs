use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user::value_objects::user_id::UserId;

pub struct RemoveUserRoleCommand {
    pub user_id: UserId,
    pub role_name: RoleName,
}
