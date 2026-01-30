use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(GitHubAccounts::Table)
                .if_not_exists()
                .col(ColumnDef::new(GitHubAccounts::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(GitHubAccounts::UserId).integer().not_null())
                .col(ColumnDef::new(GitHubAccounts::GitHubId).big_integer().not_null().unique_key())
                .col(ColumnDef::new(GitHubAccounts::Login).string_len(255).not_null().unique_key())
                .col(ColumnDef::new(GitHubAccounts::AccessToken).string_len(255))
                .col(ColumnDef::new(GitHubAccounts::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_github_user")
                        .from(GitHubAccounts::Table, GitHubAccounts::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GitHubAccounts::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum GitHubAccounts {
    Table,
    Id,
    UserId,
    GitHubId,
    Login,
    AccessToken,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
