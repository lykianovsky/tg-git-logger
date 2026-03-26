use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RepositoryPullRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RepositoryPullRequests::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::RepositoryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::PrNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::Title)
                            .string_len(512)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::Author)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::Status)
                            .string_len(32)
                            .not_null()
                            .default("open"),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryPullRequests::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    // Индекс на репозиторий и номер PR (уникальность)
                    .index(
                        Index::create()
                            .name("idx_pull_requests_repo_pr_number")
                            .col(RepositoryPullRequests::RepositoryId)
                            .col(RepositoryPullRequests::PrNumber)
                            .unique(),
                    )
                    // Внешний ключ на репозиторий
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pull_requests_repository")
                            .from(
                                RepositoryPullRequests::Table,
                                RepositoryPullRequests::RepositoryId,
                            )
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
            .drop_table(
                Table::drop()
                    .table(RepositoryPullRequests::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum RepositoryPullRequests {
    Table,
    Id,
    RepositoryId,
    PrNumber,
    Title,
    Author,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Repositories {
    Table,
    #[iden = "id"]
    Id,
}
