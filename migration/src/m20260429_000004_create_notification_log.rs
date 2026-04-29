use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotificationLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotificationLog::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NotificationLog::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(NotificationLog::Kind)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationLog::DedupKey)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationLog::SentAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notification_log_user")
                            .from(NotificationLog::Table, NotificationLog::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_notification_log_user_kind_key")
                            .col(NotificationLog::UserId)
                            .col(NotificationLog::Kind)
                            .col(NotificationLog::DedupKey),
                    )
                    .index(
                        Index::create()
                            .name("idx_notification_log_sent_at")
                            .col(NotificationLog::SentAt),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotificationLog::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum NotificationLog {
    Table,
    Id,
    UserId,
    Kind,
    DedupKey,
    SentAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
