use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Repositories::Table)
                .if_not_exists()
                .col(ColumnDef::new(Repositories::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Repositories::GitHubRepoId).big_integer().not_null().unique_key())
                .col(ColumnDef::new(Repositories::Owner).string_len(255).not_null())
                .col(ColumnDef::new(Repositories::Name).string_len(255).not_null())
                .col(ColumnDef::new(Repositories::IsActive).boolean().not_null().default(true))
                .col(ColumnDef::new(Repositories::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                .index(Index::create().name("uk_owner_name").col(Repositories::Owner).col(Repositories::Name).unique())
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Repositories::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Repositories {
    Table,
    Id,
    GitHubRepoId,
    Owner,
    Name,
    IsActive,
    CreatedAt,
}
