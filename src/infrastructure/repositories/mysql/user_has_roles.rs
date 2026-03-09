use crate::domain::role::entities::role_entity::Role;
use crate::domain::role::value_objects::role_id::RoleId;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user::repositories::user_has_roles_repository::{
    AssignRoleToUserException, GetAllUserRolesException, UserHasRolesRepository,
};
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::{roles, user_has_roles};
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use sea_orm::{ColumnTrait, QuerySelect, RelationTrait};
use sea_orm::{DatabaseTransaction, QueryFilter};
use sea_orm::{EntityTrait, Set};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLUserHasRolesRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserHasRolesRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserHasRolesRepository for MySQLUserHasRolesRepository {
    async fn assign(
        &self,
        txn: &DatabaseTransaction,
        user_id: UserId,
        role_name: RoleName,
    ) -> Result<(), AssignRoleToUserException> {
        let role = roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name.to_string()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| AssignRoleToUserException::DbError(e.to_string()))?
            .ok_or_else(|| AssignRoleToUserException::DbError("Role not found".to_string()))?;

        let active_model = user_has_roles::ActiveModel {
            user_id: Set(user_id.0),
            role_id: Set(role.id),
        };

        active_model
            .insert(txn)
            .await
            .map_err(|e| AssignRoleToUserException::DbError(e.to_string()))?;

        Ok(())
    }

    async fn get_all(&self, user_id: UserId) -> Result<Vec<Role>, GetAllUserRolesException> {
        let roles_data = roles::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                roles::Relation::UserHasRoles.def(),
            )
            .filter(user_has_roles::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await
            .map_err(|e| GetAllUserRolesException::DbError(e.to_string()))?;

        let mut roles = Vec::with_capacity(roles_data.len());

        for role in roles_data {
            let name = RoleName::from_str(&role.name)
                .map_err(|e| GetAllUserRolesException::InvalidField(e.to_string()))?;

            roles.push(Role {
                id: RoleId(role.id),
                name,
            });
        }

        Ok(roles)
    }
}
