//! `SeaORM` Entity, соответствует миграции m20260922_000002_create_test_runs

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "test_runs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub repository_id: i32,
    #[sea_orm(unique)]
    pub run_tag: String,
    pub provider_run_id: Option<i64>,
    pub run_url: Option<String>,
    pub git_ref: String,
    pub sha: Option<String>,
    pub trigger: String,
    pub args: Option<String>,
    pub requested_by_user_id: Option<i32>,
    pub chat_id: Option<i64>,
    pub message_id: Option<i32>,
    pub status: String,
    pub started_at: Option<DateTimeUtc>,
    pub finished_at: Option<DateTimeUtc>,
    pub duration_ms: Option<i64>,
    pub total: Option<i32>,
    pub passed: Option<i32>,
    pub failed: Option<i32>,
    pub flaky: Option<i32>,
    pub skipped: Option<i32>,
    pub report_state: String,
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
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::RequestedByUserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Users,
    #[sea_orm(has_many = "super::test_run_failures::Entity")]
    TestRunFailures,
}

impl Related<super::repositories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repositories.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::test_run_failures::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TestRunFailures.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
