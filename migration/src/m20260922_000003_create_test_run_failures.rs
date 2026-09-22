use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestRunFailures::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestRunFailures::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TestRunFailures::TestRunId)
                            .integer()
                            .not_null(),
                    )
                    // Проект Playwright (landing / accounts / booking-module)
                    .col(
                        ColumnDef::new(TestRunFailures::Project)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestRunFailures::File)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestRunFailures::Title)
                            .string_len(500)
                            .not_null(),
                    )
                    // Отпечаток теста — по нему ищется заведённая карточка
                    .col(
                        ColumnDef::new(TestRunFailures::Fingerprint)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(TestRunFailures::ErrorExcerpt).text().null())
                    .col(
                        ColumnDef::new(TestRunFailures::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_run_failures_run")
                            .from(TestRunFailures::Table, TestRunFailures::TestRunId)
                            .to(TestRuns::Table, TestRuns::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_test_run_failures_run")
                            .col(TestRunFailures::TestRunId),
                    )
                    .index(
                        Index::create()
                            .name("idx_test_run_failures_fingerprint")
                            .col(TestRunFailures::Fingerprint),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestRunFailures::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TestRunFailures {
    Table,
    Id,
    TestRunId,
    Project,
    File,
    Title,
    Fingerprint,
    ErrorExcerpt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum TestRuns {
    Table,
    Id,
}
