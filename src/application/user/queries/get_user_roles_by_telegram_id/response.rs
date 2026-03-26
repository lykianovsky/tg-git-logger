use crate::domain::role::value_objects::role_name::RoleName;

pub struct GetUserRolesByTelegramIdResponse {
    pub roles: Vec<RoleName>,
}
