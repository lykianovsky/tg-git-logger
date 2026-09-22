use sea_orm_migration::prelude::*;

/// Доска трекера уже выбирается при настройке репозитория, но не сохранялась.
/// Без неё нельзя создать карточку: Kaiten принимает доску вместе с колонкой
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RepositoryTaskTrackers::Table)
                    .add_column(
                        ColumnDef::new(RepositoryTaskTrackers::BoardId)
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
                    .table(RepositoryTaskTrackers::Table)
                    .drop_column(RepositoryTaskTrackers::BoardId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RepositoryTaskTrackers {
    Table,
    BoardId,
}
