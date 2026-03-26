use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Repositories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Repositories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Repositories::ExternalId)
                            .big_integer()
                            .not_null()
                            .unique_key(), // уникальный внешний ID
                    )
                    .col(
                        ColumnDef::new(Repositories::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Repositories::Owner)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Repositories::Url)
                            .string_len(1024)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Repositories::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Repositories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Repositories::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Repositories {
    Table,
    Id,
    ExternalId,
    Name,
    Owner,
    Url,
    CreatedAt,
    UpdatedAt,
}
