use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PrReviews::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PrReviews::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PrReviews::Repo).string_len(255).not_null())
                    .col(ColumnDef::new(PrReviews::PrNumber).integer().not_null())
                    .col(
                        ColumnDef::new(PrReviews::ReviewerLogin)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PrReviews::LastReviewedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PrReviews::LastReviewState)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PrReviews::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PrReviews::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("ON UPDATE CURRENT_TIMESTAMP")
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("uq_pr_reviews_repo_pr_reviewer")
                            .col(PrReviews::Repo)
                            .col(PrReviews::PrNumber)
                            .col(PrReviews::ReviewerLogin)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_pr_reviews_repo_pr")
                            .col(PrReviews::Repo)
                            .col(PrReviews::PrNumber),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PrReviews::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum PrReviews {
    Table,
    Id,
    Repo,
    PrNumber,
    ReviewerLogin,
    LastReviewedAt,
    LastReviewState,
    CreatedAt,
    UpdatedAt,
}
