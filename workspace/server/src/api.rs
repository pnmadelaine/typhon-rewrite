use axum::{
    body::Body,
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use derive_more::From;
use typhon_core::*;

type Init = init::Addr;

#[derive(From)]
enum Error {
    Init(init::Error),
    Project(projects::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        let body = match self {
            Self::Init(err) => format!("{err:?}"),
            Self::Project(err) => format!("{err:?}"),
        };
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

// init endpoints
async fn delete_project(
    Extension(mut init): Extension<Init>,
    Path(name): Path<String>,
) -> Result<(), Error> {
    Ok(init.delete_project(name).await?)
}
async fn new_project(
    Extension(mut init): Extension<Init>,
    Path(name): Path<String>,
) -> Result<(), Error> {
    Ok(init.new_project(name).await?)
}
async fn projects(Extension(mut init): Extension<Init>) -> Json<Vec<String>> {
    Json(init.projects().await)
}
async fn rename_project(
    Extension(mut init): Extension<Init>,
    Path(from): Path<String>,
    Json(to): Json<String>,
) -> Result<(), Error> {
    Ok(init.rename_project(from, to).await?)
}

// project endpoits
async fn project_disable(
    Extension(mut init): Extension<Init>,
    Path(project): Path<String>,
) -> Result<(), Error> {
    Ok(init.project(project).await?.disable().await?)
}
async fn project_enable(
    Extension(mut init): Extension<Init>,
    Path(project): Path<String>,
) -> Result<(), Error> {
    Ok(init.project(project).await?.enable().await?)
}
// async fn project_lock(
//     Extension(mut init): Extension<Init>,
//     Path(project): Path<String>,
// ) -> Result<(), Error> {
//     Ok(init.project(project).await?.lock().await?)
// }
// async fn project_lock_error(
//     Extension(mut init): Extension<Init>,
//     Path(project): Path<String>,
// ) -> Result<String, Error> {
//     Ok(init
//         .project(project)
//         .await?
//         .lock_error()
//         .await
//         .unwrap_or(String::new()))
// }
async fn project_secret_delete(
    Extension(mut init): Extension<Init>,
    Path((project, key)): Path<(String, String)>,
) -> Result<(), Error> {
    Ok(init.project(project).await?.edit_secret(key, None).await)
}
async fn project_secret_new(
    Extension(mut init): Extension<Init>,
    Path((project, key)): Path<(String, String)>,
    value: String,
) -> Result<(), Error> {
    Ok(init
        .project(project)
        .await?
        .edit_secret(key, Some(value))
        .await)
}
async fn project_secrets(
    Extension(mut init): Extension<Init>,
    Path(project): Path<String>,
) -> Result<Json<Vec<String>>, Error> {
    Ok(Json(init.project(project).await?.secrets().await))
}
async fn project_set_path(
    Extension(mut init): Extension<Init>,
    Path(project): Path<String>,
    value: String,
) -> Result<(), Error> {
    Ok(init.project(project).await?.set_path(value).await?)
}
async fn project_set_source(
    Extension(mut init): Extension<Init>,
    Path(project): Path<String>,
    Json(source): Json<Source>,
) -> Result<(), Error> {
    Ok(init.project(project).await?.set_source(source).await?)
}

pub(crate) fn api_routes<S>(init: Init) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // TODO: fallback
    use axum::routing::{delete, get, post};
    axum::Router::new()
        .route("/project/:from/rename", post(rename_project))
        .route("/project/:name", delete(delete_project))
        .route("/project/:name", post(new_project))
        .route("/project/:name/disable", post(project_disable))
        .route("/project/:name/enable", post(project_enable))
        // .route("/project/:name/lock", post(project_lock))
        // .route("/project/:name/lock_error", get(project_lock_error))
        .route("/project/:name/path", post(project_set_path))
        .route("/project/:name/secrets", get(project_secrets))
        .route("/project/:name/source", post(project_set_source))
        .route(
            "/project/:name/secrets/:key",
            post(project_secret_new).delete(project_secret_delete),
        )
        .route("/projects", get(projects))
        .layer(Extension(init))
}
