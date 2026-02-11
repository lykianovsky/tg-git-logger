use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserHasRoles::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserHasRoles::UserId).integer().not_null())
                    .col(ColumnDef::new(UserHasRoles::RoleId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(UserHasRoles::UserId)
                            .col(UserHasRoles::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_roles_user")
                            .from(UserHasRoles::Table, UserHasRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_roles_role")
                            .from(UserHasRoles::Table, UserHasRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
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
            .drop_table(Table::drop().table(UserHasRoles::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum UserHasRoles {
    Table,
    UserId,
    RoleId,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Roles {
    Table,
    Id,
}
