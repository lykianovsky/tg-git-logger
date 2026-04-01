//! `SeaORM` Entity for health_pings table

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "health_pings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub url: String,
    pub interval_minutes: i32,
    pub is_active: i8,
    pub last_checked_at: Option<DateTimeUtc>,
    pub last_status: Option<String>,
    pub last_response_ms: Option<i32>,
    pub last_error_message: Option<String>,
    pub failed_since: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
