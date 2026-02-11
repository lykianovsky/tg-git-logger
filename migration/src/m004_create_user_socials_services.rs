use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSocialsServices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSocialsServices::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialUserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialChatId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialUserLogin)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialUserEmail)
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::SocialUserAvatarUrl)
                            .string_len(512)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSocialsServices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    // ===== INDEXES =====
                    .index(
                        Index::create()
                            .name("idx_user_social_services_user_id")
                            .col(UserSocialsServices::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_social_services_provider")
                            .col(UserSocialsServices::SocialType),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_social_services_provider_username")
                            .col(UserSocialsServices::SocialType)
                            .col(UserSocialsServices::SocialUserLogin),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("unique_provider_account")
                            .col(UserSocialsServices::SocialType)
                            .col(UserSocialsServices::SocialUserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_social_services_user")
                            .from(UserSocialsServices::Table, UserSocialsServices::UserId)
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
            .drop_table(Table::drop().table(UserSocialsServices::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum UserSocialsServices {
    Table,
    Id,
    UserId,
    SocialType,
    SocialChatId,
    SocialUserId,
    SocialUserLogin,
    SocialUserEmail,
    SocialUserAvatarUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    #[iden = "id"]
    Id,
}
