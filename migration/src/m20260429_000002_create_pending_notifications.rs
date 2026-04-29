use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PendingNotifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PendingNotifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::SocialType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::SocialChatId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::Payload)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::EventType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::DeliverAfter)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingNotifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pending_notifications_user")
                            .from(PendingNotifications::Table, PendingNotifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_pending_notifications_deliver_after")
                            .col(PendingNotifications::DeliverAfter),
                    )
                    .index(
                        Index::create()
                            .name("idx_pending_notifications_user_id")
                            .col(PendingNotifications::UserId),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PendingNotifications::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum PendingNotifications {
    Table,
    Id,
    UserId,
    SocialType,
    SocialChatId,
    Payload,
    EventType,
    DeliverAfter,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
