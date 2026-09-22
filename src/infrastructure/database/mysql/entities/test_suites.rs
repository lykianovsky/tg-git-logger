//! `SeaORM` Entity, соответствует миграции m20260922_000001_create_test_suites

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "test_suites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub repository_id: i32,
    pub workflow_file: String,
    pub default_ref: String,
    pub args_input_name: String,
    pub ref_input_name: String,
    pub tag_input_name: String,
    pub artifact_prefix: String,
    pub summary_path: String,
    pub tests_root: String,
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
