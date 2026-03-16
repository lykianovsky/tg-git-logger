use crate::domain::role::value_objects::role_name::RoleName;
use crate::infrastructure::database::mysql::entities::roles;
use crate::infrastructure::database::seeder::{Seeder, SeederRunError};
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

pub struct MySQLRolesSeeder<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> MySQLRolesSeeder<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Seeder for MySQLRolesSeeder<'_> {
    async fn run(self) -> Result<(), SeederRunError> {
        tracing::debug!("Seed roles to database");

        let default_roles = vec![
            roles::Model {
                id: Default::default(),
                name: RoleName::Admin.to_string(),
            },
            roles::Model {
                id: Default::default(),
                name: RoleName::QualityAssurance.to_string(),
            },
            roles::Model {
                id: Default::default(),
                name: RoleName::Developer.to_string(),
            },
        ];

        for role in default_roles {
            let exists = roles::Entity::find()
                .filter(roles::Column::Name.eq(&role.name))
                .one(&*self.db)
                .await
                .map_err(|e| SeederRunError::DbError(e.to_string()))?;

            if exists.is_none() {
                roles::Entity::insert(role.into_active_model())
                    .exec(&*self.db)
                    .await
                    .map_err(|e| SeederRunError::InsertFailed(e.to_string()))?;
            }
        }

        tracing::debug!("Seed roles to database complete successfully");

        Ok(())
    }
}
