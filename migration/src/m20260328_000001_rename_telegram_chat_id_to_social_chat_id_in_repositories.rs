use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .rename_column(Repositories::TelegramChatId, Repositories::SocialChatId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Repositories::Table)
                    .rename_column(Repositories::SocialChatId, Repositories::TelegramChatId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    TelegramChatId,
    SocialChatId,
}
