//! `SeaORM` Entity for digest_subscriptions table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "digest_subscriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub repository_id: Option<i32>,
    pub digest_type: String,
    pub send_hour: i8,
    pub send_minute: i8,
    pub day_of_week: Option<i8>,
    pub is_active: i8,
    pub last_sent_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Users,

    #[sea_orm(
        belongs_to = "super::repositories::Entity",
        from = "Column::RepositoryId",
        to = "super::repositories::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Repositories,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::repositories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
