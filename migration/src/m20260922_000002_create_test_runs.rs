use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestRuns::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestRuns::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TestRuns::RepositoryId).integer().not_null())
                    // Метка запуска: её бот кладёт во вход workflow и по ней находит свой прогон
                    .col(
                        ColumnDef::new(TestRuns::RunTag)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    // Идентификатор прогона в CI — известен после старта
                    .col(ColumnDef::new(TestRuns::ProviderRunId).big_integer().null())
                    .col(ColumnDef::new(TestRuns::RunUrl).string_len(500).null())
                    .col(ColumnDef::new(TestRuns::GitRef).string_len(255).not_null())
                    .col(ColumnDef::new(TestRuns::Sha).string_len(64).null())
                    // chat / schedule / deploy — чем запущен прогон
                    .col(
                        ColumnDef::new(TestRuns::Trigger)
                            .string_len(32)
                            .not_null()
                            .default("chat"),
                    )
                    .col(ColumnDef::new(TestRuns::Args).string_len(1000).null())
                    .col(ColumnDef::new(TestRuns::RequestedByUserId).integer().null())
                    // Сообщение-карточка в чате, которое обновляется по ходу прогона
                    .col(ColumnDef::new(TestRuns::ChatId).big_integer().null())
                    .col(ColumnDef::new(TestRuns::MessageId).integer().null())
                    .col(
                        ColumnDef::new(TestRuns::Status)
                            .string_len(32)
                            .not_null()
                            .default("queued"),
                    )
                    .col(
                        ColumnDef::new(TestRuns::StartedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TestRuns::FinishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(TestRuns::DurationMs).big_integer().null())
                    .col(ColumnDef::new(TestRuns::Total).integer().null())
                    .col(ColumnDef::new(TestRuns::Passed).integer().null())
                    .col(ColumnDef::new(TestRuns::Failed).integer().null())
                    .col(ColumnDef::new(TestRuns::Flaky).integer().null())
                    .col(ColumnDef::new(TestRuns::Skipped).integer().null())
                    .col(
                        ColumnDef::new(TestRuns::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestRuns::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_runs_repository")
                            .from(TestRuns::Table, TestRuns::RepositoryId)
                            .to(Repositories::Table, Repositories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_runs_user")
                            .from(TestRuns::Table, TestRuns::RequestedByUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Последний прогон репозитория и поиск активного
                    .index(
                        Index::create()
                            .name("idx_test_runs_repository_created")
                            .col(TestRuns::RepositoryId)
                            .col(TestRuns::CreatedAt),
                    )
                    .index(
                        Index::create()
                            .name("idx_test_runs_repository_status")
                            .col(TestRuns::RepositoryId)
                            .col(TestRuns::Status),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestRuns::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TestRuns {
    Table,
    Id,
    RepositoryId,
    RunTag,
    ProviderRunId,
    RunUrl,
    GitRef,
    Sha,
    Trigger,
    Args,
    RequestedByUserId,
    ChatId,
    MessageId,
    Status,
    StartedAt,
    FinishedAt,
    DurationMs,
    Total,
    Passed,
    Failed,
    Flaky,
    Skipped,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
