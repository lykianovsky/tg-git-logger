use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DigestSubscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DigestSubscriptions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::RepositoryId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::DigestType)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::SendHour)
                            .tiny_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::SendMinute)
                            .tiny_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::DayOfWeek)
                            .tiny_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::IsActive)
                            .tiny_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::LastSentAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DigestSubscriptions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_digest_subscriptions_user")
                            .from(DigestSubscriptions::Table, DigestSubscriptions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_digest_subscriptions_repository")
                            .from(
                                DigestSubscriptions::Table,
                                DigestSubscriptions::RepositoryId,
                            )
                            .to(Repositories::Table, Repositories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_digest_subscriptions_user_id")
                            .col(DigestSubscriptions::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_digest_subscriptions_active_hour_minute")
                            .col(DigestSubscriptions::IsActive)
                            .col(DigestSubscriptions::SendHour)
                            .col(DigestSubscriptions::SendMinute),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DigestSubscriptions::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum DigestSubscriptions {
    Table,
    Id,
    UserId,
    RepositoryId,
    DigestType,
    SendHour,
    SendMinute,
    DayOfWeek,
    IsActive,
    LastSentAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}
