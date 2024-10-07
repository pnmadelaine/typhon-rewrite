use crate::prelude::*;

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::builds)]
pub(super) struct NewBuild<'a> {
    pub(super) id: i64,
    pub(super) drv: &'a str,
    pub(super) started_at: SystemTime,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::builds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Build {
    pub(super) id: i64,
    pub(super) drv: String,
    pub(super) started_at: SystemTime,
    pub(super) ended_at: Option<SystemTime>,
    pub(super) success: Option<bool>,
}
