use sea_orm_migration::prelude::*;

/// Колонка, в которую бот кладёт новые задачи — карточки по упавшим тестам.
/// Это не колонка QA: туда карточки переезжают при мерже, а новые заводятся отдельно
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RepositoryTaskTracker::Table)
                    .add_column(
                        ColumnDef::new(RepositoryTaskTracker::NewTaskColumnId)
                            .integer()
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
                    .table(RepositoryTaskTracker::Table)
                    .drop_column(RepositoryTaskTracker::NewTaskColumnId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RepositoryTaskTracker {
    #[sea_orm(iden = "repository_task_tracker")]
    Table,
    NewTaskColumnId,
}
