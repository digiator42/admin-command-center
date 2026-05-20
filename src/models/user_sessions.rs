// 📁 production_app: src/models/user_sessions.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String, // The secure token string
    pub user_id: i32,
    pub expires_at: DateTime,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::system_users::Entity",
        from = "Column::UserId",
        to = "super::system_users::Column::Id"
    )]
    SystemUsers,
}

impl Related<super::system_users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SystemUsers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}