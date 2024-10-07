use crate::{models, prelude::*, result::*};

pub(crate) struct Permissions {
    admin: bool,
    projects: Vec<i32>,
}

pub(crate) struct TokenInfo {
    name: String,
    created_at: SystemTime,
    valid_until: SystemTime,
    last_used_at: Option<SystemTime>,
    revoked_at: Option<SystemTime>,
}

async fn set_admin(conn: &mut Conn<'_>, username: String, admin: bool) -> InternalResult<()> {
    diesel::update(schema::users::table)
        .filter(schema::users::username.eq(username))
        .set(schema::users::is_super_admin.eq(admin))
        .execute(conn)
        .await?;
    Ok(())
}

pub(crate) async fn make_admin(conn: &mut Conn<'_>, username: String) -> InternalResult<()> {
    set_admin(conn, username, true).await
}

pub(crate) async fn revoke_admin(conn: &mut Conn<'_>, username: String) -> InternalResult<()> {
    set_admin(conn, username, false).await
}

pub(crate) async fn grant_permission(
    conn: &mut Conn<'_>,
    username: String,
    project: String,
) -> InternalResult<()> {
    let user_id = schema::users::table
        .filter(schema::users::username.eq(username))
        .select(schema::users::id)
        .first(conn)
        .await?;
    let project_id = schema::projects::table
        .filter(schema::projects::name.eq(project))
        .select(schema::projects::id)
        .first(conn)
        .await?;
    let permission = models::NewPermission {
        user_id,
        project_id,
    };
    diesel::insert_into(schema::permissions::table)
        .values(permission)
        .execute(conn)
        .await?;
    Ok(())
}

pub(crate) async fn revoke_permission(
    conn: &mut Conn<'_>,
    username: String,
    project: String,
) -> InternalResult<()> {
    let id: i32 = schema::permissions::table
        .inner_join(schema::users::table)
        .inner_join(schema::projects::table)
        .filter(schema::users::username.eq(username))
        .filter(schema::projects::name.eq(project))
        .select(schema::permissions::id)
        .first(conn)
        .await?;
    diesel::delete(schema::permissions::table.filter(schema::permissions::id.eq(id)))
        .execute(conn)
        .await?;
    Ok(())
}

pub(crate) async fn delete_user(conn: &mut Conn<'_>, username: String) -> InternalResult<()> {
    let id: i32 = schema::users::table
        .filter(schema::users::username.eq(username))
        .select(schema::users::id)
        .first(conn)
        .await?;
    diesel::delete(schema::permissions::table.filter(schema::permissions::user_id.eq(id)))
        .execute(conn)
        .await?;
    diesel::delete(schema::users::table.filter(schema::users::id.eq(id)))
        .execute(conn)
        .await?;
    Ok(())
}
