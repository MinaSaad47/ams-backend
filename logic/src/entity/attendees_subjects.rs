//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Default)]
#[sea_orm(table_name = "attendees_subjects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub attendee_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub subject_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::attendees::Entity",
        from = "Column::AttendeeId",
        to = "super::attendees::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Attendees,
    #[sea_orm(
        belongs_to = "super::subjects::Entity",
        from = "Column::SubjectId",
        to = "super::subjects::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Subjects,
}

impl Related<super::attendees::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attendees.def()
    }
}

impl Related<super::subjects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subjects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
