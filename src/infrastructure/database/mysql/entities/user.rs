use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    TelegramAccount,
    GithubAccount,
    UserRole,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::TelegramAccount => Entity::has_one(super::telegram_account::Entity).into(),
            Self::GithubAccount => Entity::has_one(super::github_account::Entity).into(),
            Self::UserRole => Entity::has_many(super::user_role::Entity).into(),
        }
    }
}

impl Related<super::telegram_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TelegramAccount.def()
    }
}

impl Related<super::github_account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GithubAccount.def()
    }
}

impl Related<super::user_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
