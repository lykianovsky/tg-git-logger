use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserNotifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserNotifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::UserId)
                            .integer()
                            .not_null(),
                    )
                    // Discriminant: "pull_request", extensible for "issues", "comments", etc.
                    .col(
                        ColumnDef::new(UserNotifications::NotificationType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::IntervalMinutes)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::IsActive)
                            .tiny_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::LastNotifiedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserNotifications::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_notifications_user_id")
                            .col(UserNotifications::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_notifications_is_active_type")
                            .col(UserNotifications::IsActive)
                            .col(UserNotifications::NotificationType),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_notifications_user")
                            .from(UserNotifications::Table, UserNotifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserNotifications::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum UserNotifications {
    Table,
    Id,
    UserId,
    NotificationType,
    IntervalMinutes,
    IsActive,
    LastNotifiedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    #[iden = "id"]
    Id,
}
