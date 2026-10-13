use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "monitored_services")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub target_url: String,
    pub expected_status: i16,
    pub ping_interval_seconds: i32,
    pub current_status: String,
    pub updated_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::service_heartbeats::Entity")]
    ServiceHeartbeats,
}

impl Related<super::service_heartbeats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceHeartbeats.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
