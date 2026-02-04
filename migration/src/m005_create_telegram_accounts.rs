use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(TelegramAccounts::Table)
                .if_not_exists()
                .col(ColumnDef::new(TelegramAccounts::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(TelegramAccounts::UserId).integer().not_null())
                .col(ColumnDef::new(TelegramAccounts::TelegramUserId).big_integer().not_null().unique_key())
                .col(ColumnDef::new(TelegramAccounts::Username).string_len(255))
                .col(ColumnDef::new(TelegramAccounts::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_telegram_user")
                        .from(TelegramAccounts::Table, TelegramAccounts::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(TelegramAccounts::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum TelegramAccounts {
    Table,
    Id,
    UserId,
    TelegramUserId,
    Username,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
