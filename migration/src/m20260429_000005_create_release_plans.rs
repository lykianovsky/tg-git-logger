use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ReleasePlans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReleasePlans::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReleasePlans::PlannedDate).date().not_null())
                    .col(
                        ColumnDef::new(ReleasePlans::CallDatetime)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::MeetingUrl)
                            .string_len(500)
                            .null(),
                    )
                    .col(ColumnDef::new(ReleasePlans::Note).text().null())
                    .col(
                        ColumnDef::new(ReleasePlans::Status)
                            .string_len(32)
                            .not_null()
                            .default("planned"),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::AnnounceChatId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::Notified24hAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::NotifiedCallAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::NotifiedReleaseDayAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::CreatedByUserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlans::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_release_plans_user")
                            .from(ReleasePlans::Table, ReleasePlans::CreatedByUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_release_plans_status_date")
                            .col(ReleasePlans::Status)
                            .col(ReleasePlans::PlannedDate),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReleasePlans::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ReleasePlans {
    Table,
    Id,
    PlannedDate,
    CallDatetime,
    MeetingUrl,
    Note,
    Status,
    AnnounceChatId,
    Notified24hAt,
    NotifiedCallAt,
    NotifiedReleaseDayAt,
    CreatedByUserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
