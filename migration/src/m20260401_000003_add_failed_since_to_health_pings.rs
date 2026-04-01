use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(HealthPings::Table)
                    .add_column(
                        ColumnDef::new(HealthPings::FailedSince)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(HealthPings::Table)
                    .drop_column(HealthPings::FailedSince)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum HealthPings {
    Table,
    FailedSince,
}
