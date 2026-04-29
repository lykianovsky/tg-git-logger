use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ReleasePlanRepositories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReleasePlanRepositories::ReleasePlanId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ReleasePlanRepositories::RepositoryId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ReleasePlanRepositories::ReleasePlanId)
                            .col(ReleasePlanRepositories::RepositoryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rpr_release_plan")
                            .from(
                                ReleasePlanRepositories::Table,
                                ReleasePlanRepositories::ReleasePlanId,
                            )
                            .to(ReleasePlans::Table, ReleasePlans::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rpr_repository")
                            .from(
                                ReleasePlanRepositories::Table,
                                ReleasePlanRepositories::RepositoryId,
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
                    .table(ReleasePlanRepositories::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ReleasePlanRepositories {
    Table,
    ReleasePlanId,
    RepositoryId,
}

#[derive(DeriveIden)]
enum ReleasePlans {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}
