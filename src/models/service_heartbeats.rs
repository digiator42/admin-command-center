use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "service_heartbeats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub service_id: i32,
    pub latency_ms: i32,
    pub http_status_code: i16,
    pub status_msg: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub error_log: Option<String>,
    pub checked_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::monitored_services::Entity",
        from = "Column::ServiceId",
        to = "super::monitored_services::Column::Id"
    )]
    MonitoredServices,
}

impl Related<super::monitored_services::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MonitoredServices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
