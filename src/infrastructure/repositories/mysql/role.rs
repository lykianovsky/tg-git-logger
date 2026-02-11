use crate::domain::role::entities::role_entity::Role;
use crate::domain::role::repositories::role_repository::{
    CreateRoleException, FindRoleByIdException, RoleRepository,
};
use crate::domain::role::value_objects::role_id::RoleId;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::infrastructure::database::mysql::entities::{roles, users};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLRoleRepository {
    pub db: Arc<DatabaseConnection>,
}

impl Role {
    pub fn from_mysql(mysql_role: roles::Model) -> Result<Self, String> {
        Ok(Self {
            id: RoleId(mysql_role.id),
            name: RoleName::from_str(&mysql_role.name)?,
        })
    }
}

impl MySQLRoleRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoleRepository for MySQLRoleRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        role: &Role,
    ) -> Result<(), CreateRoleException> {
        let role_model = roles::ActiveModel {
            name: Set(role.name.to_string()),
            ..Default::default()
        };

        role_model
            .insert(txn)
            .await
            .map_err(|e| CreateRoleException::DbError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: RoleId) -> Result<Role, FindRoleByIdException> {
        let role = roles::Entity::find()
            .filter(users::Column::Id.eq(id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRoleByIdException::DbError(e.to_string()))?
            .ok_or(FindRoleByIdException::NotFound)?;

        Ok(Role::from_mysql(role).map_err(|e| FindRoleByIdException::DbError(e.to_string()))?)
    }
}
