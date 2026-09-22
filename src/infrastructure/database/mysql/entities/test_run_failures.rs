//! `SeaORM` Entity, соответствует миграции m20260922_000003_create_test_run_failures

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "test_run_failures")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub test_run_id: i32,
    pub project: String,
    pub file: String,
    pub title: String,
    pub fingerprint: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub error_excerpt: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::test_runs::Entity",
        from = "Column::TestRunId",
        to = "super::test_runs::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    TestRuns,
}

impl Related<super::test_runs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TestRuns.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
