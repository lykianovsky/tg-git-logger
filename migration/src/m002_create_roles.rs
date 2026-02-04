use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Roles::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Roles::Name).string_len(64).not_null().unique_key())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Roles::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
}
