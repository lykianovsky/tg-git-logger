use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RepositoryTaskTracker::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::RepositoryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::SpaceId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::QaColumnId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::ExtractPatternRegexp)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::PathToCard)
                            .string_len(1024)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RepositoryTaskTracker::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_task_tracker_repository_id_unique")
                            .col(RepositoryTaskTracker::RepositoryId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_tracker_repository")
                            .from(
                                RepositoryTaskTracker::Table,
                                RepositoryTaskTracker::RepositoryId,
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
            .drop_table(Table::drop().table(RepositoryTaskTracker::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RepositoryTaskTracker {
    Table,
    Id,
    RepositoryId,
    SpaceId,
    QaColumnId,
    ExtractPatternRegexp,
    PathToCard,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}
