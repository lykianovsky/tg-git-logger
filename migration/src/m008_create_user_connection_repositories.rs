use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserConnectionRepositories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserConnectionRepositories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserConnectionRepositories::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConnectionRepositories::RepositoryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConnectionRepositories::IsActive)
                            .tiny_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(UserConnectionRepositories::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConnectionRepositories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    // Индексы
                    .index(
                        Index::create()
                            .name("idx_user_conn_repos_user_id")
                            .col(UserConnectionRepositories::UserId),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_conn_repos_user_repo")
                            .col(UserConnectionRepositories::UserId)
                            .col(UserConnectionRepositories::RepositoryId),
                    )
                    // Внешний ключ на пользователя
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_conn_repos_user")
                            .from(
                                UserConnectionRepositories::Table,
                                UserConnectionRepositories::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Внешний ключ на репозиторий (предположим, что таблица Repositories существует)
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_conn_repos_repository")
                            .from(
                                UserConnectionRepositories::Table,
                                UserConnectionRepositories::RepositoryId,
                            )
                            .to(Repositories::Table, Repositories::Id)
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
            .drop_table(
                Table::drop()
                    .table(UserConnectionRepositories::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum UserConnectionRepositories {
    Table,
    Id,
    UserId,
    RepositoryId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    #[iden = "id"]
    Id,
}

#[derive(Iden)]
enum Repositories {
    Table,
    #[iden = "id"]
    Id,
}
