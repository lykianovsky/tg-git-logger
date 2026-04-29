use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop FK first (имя из m20260429_000002), затем меняем на nullable, затем восстанавливаем FK.
        manager
            .alter_table(
                Table::alter()
                    .table(PendingNotifications::Table)
                    .drop_foreign_key(Alias::new("fk_pending_notifications_user"))
                    .to_owned(),
            )
            .await
            .ok();

        manager
            .alter_table(
                Table::alter()
                    .table(PendingNotifications::Table)
                    .modify_column(
                        ColumnDef::new(PendingNotifications::UserId)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PendingNotifications::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_pending_notifications_user")
                            .from_tbl(PendingNotifications::Table)
                            .from_col(PendingNotifications::UserId)
                            .to_tbl(Users::Table)
                            .to_col(Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
            .ok();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PendingNotifications::Table)
                    .modify_column(
                        ColumnDef::new(PendingNotifications::UserId)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum PendingNotifications {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
