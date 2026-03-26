//! `SeaORM` Entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "repository_task_tracker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub repository_id: i32,
    pub space_id: i32,
    pub qa_column_id: i32,
    pub extract_pattern_regexp: String,
    pub path_to_card: String,
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
