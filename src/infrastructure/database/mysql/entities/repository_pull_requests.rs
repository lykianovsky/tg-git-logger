//! `SeaORM` Entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "repository_pull_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub repository_id: i32,
    pub pr_number: i32,
    pub title: String,
    pub author: String,
    pub status: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::repositories::Entity",
        from = "Column::RepositoryId",
        to = "super::repositories::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Repositories,
}

impl Related<super::repositories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
