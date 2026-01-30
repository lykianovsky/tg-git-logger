use crate::sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ------------------ 1️⃣ users ------------------
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await?;

        // ------------------ 2️⃣ roles ------------------
        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Roles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Roles::Name).string().not_null())
                    .col(
                        ColumnDef::new(Roles::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Roles::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await?;

        // ------------------ 3️⃣ user_roles (Many-to-Many) ------------------
        manager
            .create_table(
                Table::create()
                    .table(UserRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserRoles::UserId).integer().not_null())
                    .col(ColumnDef::new(UserRoles::RoleId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userroles_user")
                            .from(UserRoles::Table, UserRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_userroles_role")
                            .from(UserRoles::Table, UserRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_roles_user_id_role_id")
                            .col(UserRoles::UserId)
                            .col(UserRoles::RoleId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // ------------------ 4️⃣ telegram_accounts ------------------
        manager
            .create_table(
                Table::create()
                    .table(TelegramAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramAccounts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TelegramAccounts::UserId).integer().not_null())
                    .col(ColumnDef::new(TelegramAccounts::TelegramId).big_integer().not_null())
                    .col(ColumnDef::new(TelegramAccounts::Username).string().null())
                    .col(ColumnDef::new(TelegramAccounts::ChatId).big_integer().not_null())
                    .col(
                        ColumnDef::new(TelegramAccounts::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TelegramAccounts::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_telegram_user")
                            .from(TelegramAccounts::Table, TelegramAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_telegram_accounts_user_id")
                            .col(TelegramAccounts::UserId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_telegram_accounts_telegram_id")
                            .col(TelegramAccounts::TelegramId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // ------------------ 5️⃣ github_accounts ------------------
        manager
            .create_table(
                Table::create()
                    .table(GithubAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GithubAccounts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GithubAccounts::UserId).integer().not_null())
                    .col(ColumnDef::new(GithubAccounts::GithubId).big_integer().not_null())
                    .col(ColumnDef::new(GithubAccounts::Login).string().not_null())
                    .col(
                        ColumnDef::new(GithubAccounts::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(GithubAccounts::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp())
                            .extra("ON UPDATE CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_github_user")
                            .from(GithubAccounts::Table, GithubAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_github_accounts_user_id")
                            .col(GithubAccounts::UserId)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_github_accounts_github_id")
                            .col(GithubAccounts::GithubId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GithubAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TelegramAccounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum UserRoles {
    Table,
    Id,
    UserId,
    RoleId,
}

#[derive(Iden)]
enum TelegramAccounts {
    Table,
    Id,
    UserId,
    TelegramId,
    Username,
    ChatId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum GithubAccounts {
    Table,
    Id,
    UserId,
    GithubId,
    Login,
    CreatedAt,
    UpdatedAt,
}