use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1) add column as nullable
        manager
            .alter_table(
                Table::alter()
                    .table(RepositoryTaskTracker::Table)
                    .add_column(
                        ColumnDef::new(RepositoryTaskTracker::ReviewColumnId)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 2) backfill existing rows: review_column_id := qa_column_id
        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE repository_task_tracker SET review_column_id = qa_column_id WHERE review_column_id IS NULL",
        )
        .await?;

        // 3) make column NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(RepositoryTaskTracker::Table)
                    .modify_column(
                        ColumnDef::new(RepositoryTaskTracker::ReviewColumnId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RepositoryTaskTracker::Table)
                    .drop_column(RepositoryTaskTracker::ReviewColumnId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum RepositoryTaskTracker {
    Table,
    ReviewColumnId,
}
