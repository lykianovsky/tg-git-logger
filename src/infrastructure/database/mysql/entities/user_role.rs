use sea_orm::entity::prelude::*;
// <- обязательно

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
}

// связи с другими таблицами
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User,
    Role,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::User => Entity::belongs_to(super::user::Entity)
                .from(Column::UserId)
                .to(super::user::Column::Id)
                .into(),
            Relation::Role => Entity::belongs_to(super::roles::Entity)
                .from(Column::RoleId)
                .to(super::roles::Column::Id)
                .into(),
        }
    }
}

// имплементации Related, чтобы find_also_related работал
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

// SeaORM ActiveModel поведение
impl ActiveModelBehavior for ActiveModel {}
