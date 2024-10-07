use crate::prelude::*;

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) is_super_admin: bool,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::users)]
pub(crate) struct NewUser<'a> {
    pub(crate) username: &'a str,
    pub(crate) is_super_admin: bool,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = schema::permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Permission {
    pub(crate) id: i32,
    pub(crate) user_id: i32,
    pub(crate) project_id: i32,
    pub(crate) admin: bool,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = schema::permissions)]
pub(crate) struct NewPermission {
    pub(crate) user_id: i32,
    pub(crate) project_id: i32,
}
