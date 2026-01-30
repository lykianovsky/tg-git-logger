use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(RepositorySettings::Table)
                .if_not_exists()
                .col(ColumnDef::new(RepositorySettings::RepositoryId).integer().not_null().primary_key())
                // Task Tracker
                .col(ColumnDef::new(RepositorySettings::TaskTrackerApiToken).string_len(255).not_null())
                .col(ColumnDef::new(RepositorySettings::TaskTrackerBase).string_len(255).not_null())
                .col(ColumnDef::new(RepositorySettings::TaskTrackerSpaceId).big_integer().not_null())
                .col(ColumnDef::new(RepositorySettings::TaskTrackerQaColumnId).big_integer().not_null())
                .col(ColumnDef::new(RepositorySettings::TaskTrackerRegexp).string_len(255).not_null())
                .col(ColumnDef::new(RepositorySettings::TaskTrackerPathToCard).string_len(255).not_null())
                // Telegram
                .col(ColumnDef::new(RepositorySettings::TelegramChatId).big_integer().not_null())
                // GitHub
                .col(ColumnDef::new(RepositorySettings::GitHubWebhookSecret).string_len(255).not_null())
                .col(ColumnDef::new(RepositorySettings::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()).extra("ON UPDATE CURRENT_TIMESTAMP".to_string()))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_repo_settings_repo")
                        .from(RepositorySettings::Table, RepositorySettings::RepositoryId)
                        .to(Repositories::Table, Repositories::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RepositorySettings::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum RepositorySettings {
    Table,
    RepositoryId,
    TaskTrackerApiToken,
    TaskTrackerBase,
    TaskTrackerSpaceId,
    TaskTrackerQaColumnId,
    TaskTrackerRegexp,
    TaskTrackerPathToCard,
    TelegramChatId,
    GitHubWebhookSecret,
    UpdatedAt,
}

#[derive(Iden)]
enum Repositories {
    Table,
    Id,
}
