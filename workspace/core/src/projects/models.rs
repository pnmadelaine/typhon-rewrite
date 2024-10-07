use crate::prelude::*;

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::projects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct Project {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) source: Option<String>,
    pub(super) path: Option<String>,
    pub(super) enabled: bool,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::projects)]
pub(super) struct NewProject<'a> {
    pub(super) name: &'a str,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::project_locks)]
pub(super) struct NewProjectLock<'a> {
    pub(super) project_id: i32,
    pub(super) lock: &'a str,
    pub(super) out_path: &'a str,
    pub(super) created_at: SystemTime,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::secrets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct Secret {
    pub(super) id: i32,
    pub(super) key: String,
    pub(super) value: String,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::secrets)]
pub(super) struct NewSecret<'a> {
    pub(super) project_id: i32,
    pub(super) key: &'a str,
    pub(super) value: &'a str,
}
