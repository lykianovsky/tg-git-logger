use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestSuites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestSuites::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::RepositoryId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::WorkflowFile)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::DefaultRef)
                            .string_len(255)
                            .not_null()
                            .default("dev"),
                    )
                    .col(
                        ColumnDef::new(TestSuites::ArgsInputName)
                            .string_len(64)
                            .not_null()
                            .default("e2e"),
                    )
                    .col(
                        ColumnDef::new(TestSuites::RefInputName)
                            .string_len(64)
                            .not_null()
                            .default("ref"),
                    )
                    .col(
                        ColumnDef::new(TestSuites::TagInputName)
                            .string_len(64)
                            .not_null()
                            .default("run_tag"),
                    )
                    .col(
                        ColumnDef::new(TestSuites::ArtifactPrefix)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::SummaryPath)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::ReportPath)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::TestsRoot)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestSuites::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_suites_repository")
                            .from(TestSuites::Table, TestSuites::RepositoryId)
                            .to(Repositories::Table, Repositories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestSuites::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TestSuites {
    Table,
    Id,
    RepositoryId,
    WorkflowFile,
    DefaultRef,
    ArgsInputName,
    RefInputName,
    TagInputName,
    ArtifactPrefix,
    SummaryPath,
    ReportPath,
    TestsRoot,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}
