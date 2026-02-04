use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(RepositoryUsers::Table)
                .if_not_exists()
                .col(ColumnDef::new(RepositoryUsers::RepositoryId).integer().not_null())
                .col(ColumnDef::new(RepositoryUsers::UserId).integer().not_null())
                .col(ColumnDef::new(RepositoryUsers::Role).enumeration("roles", [ "admin", "qa", "viewer" ]).not_null().default("viewer"))
                .col(ColumnDef::new(RepositoryUsers::AddedAt).timestamp().not_null().default(Expr::current_timestamp()))
                .primary_key(Index::create().col(RepositoryUsers::RepositoryId).col(RepositoryUsers::UserId))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_repo_users_repo")
                        .from(RepositoryUsers::Table, RepositoryUsers::RepositoryId)
                        .to(Repositories::Table, Repositories::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_repo_users_user")
                        .from(RepositoryUsers::Table, RepositoryUsers::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(RepositoryUsers::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum RepositoryUsers {
    Table,
    RepositoryId,
    UserId,
    Role,
    AddedAt,
}

#[derive(Iden)]
enum Repositories {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
