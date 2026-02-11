use crate::domain::role::value_objects::role_id::RoleId;
use crate::domain::role::value_objects::role_name::RoleName;

#[derive(Debug, Clone)]
pub struct Role {
    pub id: RoleId,
    pub name: RoleName,
}
