use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HealthPings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HealthPings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::Url)
                            .string_len(1024)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::IntervalMinutes)
                            .integer()
                            .not_null()
                            .default(5),
                    )
                    .col(
                        ColumnDef::new(HealthPings::IsActive)
                            .tiny_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(HealthPings::LastCheckedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::LastStatus)
                            .string_len(16)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::LastResponseMs)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::LastErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(HealthPings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_health_pings_is_active")
                            .col(HealthPings::IsActive),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HealthPings::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum HealthPings {
    Table,
    Id,
    Name,
    Url,
    IntervalMinutes,
    IsActive,
    LastCheckedAt,
    LastStatus,
    LastResponseMs,
    LastErrorMessage,
    CreatedAt,
    UpdatedAt,
}
