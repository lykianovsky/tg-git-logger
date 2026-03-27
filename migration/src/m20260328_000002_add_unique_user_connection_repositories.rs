use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the non-unique composite index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_conn_repos_user_repo")
                    .table(UserConnectionRepositories::Table)
                    .to_owned(),
            )
            .await?;

        // Re-create it as a unique constraint
        manager
            .create_index(
                Index::create()
                    .name("uq_user_conn_repos_user_repo")
                    .table(UserConnectionRepositories::Table)
                    .col(UserConnectionRepositories::UserId)
                    .col(UserConnectionRepositories::RepositoryId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_user_conn_repos_user_repo")
                    .table(UserConnectionRepositories::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_conn_repos_user_repo")
                    .table(UserConnectionRepositories::Table)
                    .col(UserConnectionRepositories::UserId)
                    .col(UserConnectionRepositories::RepositoryId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum UserConnectionRepositories {
    Table,
    UserId,
    RepositoryId,
}
