use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestFailureCards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestFailureCards::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::RepositoryId)
                            .integer()
                            .not_null(),
                    )
                    // Отпечаток теста: репозиторий + проект + файл + название
                    .col(
                        ColumnDef::new(TestFailureCards::Fingerprint)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::Project)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::File)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::Title)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::CardId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::CardUrl)
                            .string_len(500)
                            .not_null(),
                    )
                    // Карточка, заведённая по этому же тесту раньше и уже закрытая
                    .col(
                        ColumnDef::new(TestFailureCards::PreviousCardId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::CreatedByUserId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TestFailureCards::ClosedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_failure_cards_repository")
                            .from(TestFailureCards::Table, TestFailureCards::RepositoryId)
                            .to(Repositories::Table, Repositories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_test_failure_cards_user")
                            .from(TestFailureCards::Table, TestFailureCards::CreatedByUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Один тест — одна активная карточка
                    .index(
                        Index::create()
                            .name("uq_test_failure_cards_repository_fingerprint")
                            .col(TestFailureCards::RepositoryId)
                            .col(TestFailureCards::Fingerprint)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestFailureCards::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum TestFailureCards {
    Table,
    Id,
    RepositoryId,
    Fingerprint,
    Project,
    File,
    Title,
    CardId,
    CardUrl,
    PreviousCardId,
    CreatedByUserId,
    CreatedAt,
    ClosedAt,
}

#[derive(DeriveIden)]
enum Repositories {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
